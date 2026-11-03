//! Detection of descending tonal ridges (aircraft Doppler signatures) and mains hum.
//!
//! Output is deliberately limited to *what is in the clip* (counts, times, frequencies,
//! a qualitative exposure level). Nothing here estimates or outputs a place.
use crate::stft::Stft;

#[derive(Clone, Debug)]
pub struct Ridge { pub start_frame: usize, pub freqs: Vec<f32> }
impl Ridge {
    pub fn end_frame(&self) -> usize { self.start_frame + self.freqs.len() - 1 }
}

#[derive(Clone, Debug)]
pub struct Flyover { pub start_s: f32, pub end_s: f32, pub peak_s: f32, pub f_start: f32, pub f_end: f32, pub ridges: Vec<Ridge> }

#[derive(Clone, Debug, Default)]
pub struct Hum { pub detected: bool, pub base_hz: f32, pub harmonics: Vec<f32> }

#[derive(Clone, Debug)]
pub struct Report {
    pub duration_s: f32,
    pub flyovers: Vec<Flyover>,
    pub hum: Hum,
    /// 0–100 exposure score.
    pub score: u32,
    pub level: Level,
    /// Relative "how narrow" rating only; never a place.
    pub narrowness: Narrowness,
    pub notes: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level { Low, Medium, High }
impl Level {
    fn from_score(score: u32) -> Self { if score >= 60 { Self::High } else if score >= 25 { Self::Medium } else { Self::Low } }
    pub fn as_str(self) -> &'static str { match self { Self::Low => "low", Self::Medium => "medium", Self::High => "high" } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Narrowness { Low, Moderate, High }
impl Narrowness {
    pub fn as_str(self) -> &'static str { match self { Self::Low => "low", Self::Moderate => "moderate", Self::High => "high" } }
}

const BAND_LO: f32 = 60.0;
const BAND_HI: f32 = 2000.0;

fn near_hum(f: f32, base: f32, tol: f32) -> bool {
    if base <= 0.0 { return false; }
    let k = (f / base).round(); (1.0..=20.0).contains(&k) && (f - k * base).abs() < tol
}

/// Parabolic-interpolated sub-bin peak frequency.
fn interp(m: &[f32], k: usize, hz: f32) -> f32 {
    let (a, b, c) = (m[k - 1].ln(), m[k].ln(), m[k + 1].ln());
    let d = a - 2.0 * b + c;
    let p = if d.abs() > 1e-9 { 0.5 * (a - c) / d } else { 0.0 };
    (k as f32 + p.clamp(-0.5, 0.5)) * hz
}

pub fn find_ridges(st: &Stft, hum_base: f32) -> Vec<Ridge> {
    let min_len = (2.0 * st.sr / st.hop as f32) as usize;
    let mut raw = find_ridges_raw(st, hum_base);
    raw.sort_by_key(|r| r.start_frame);
    // Stitch fragments of one ridge that were broken by crossings or fades.
    let max_gap = (3.0 * st.sr / st.hop as f32) as usize;
    let mut merged: Vec<Ridge> = vec![];
    for r in raw {
        let f0 = r.freqs[0];
        let cand = merged.iter_mut().filter(|m| m.end_frame() < r.start_frame && r.start_frame - m.end_frame() <= max_gap
            && (*m.freqs.last().unwrap() - f0).abs() < 0.06 * f0 && *m.freqs.last().unwrap() >= f0 - 2.0)
            .min_by(|a, b| (*a.freqs.last().unwrap() - f0).abs().partial_cmp(&(*b.freqs.last().unwrap() - f0).abs()).unwrap());
        if let Some(m) = cand {
            let prev = *m.freqs.last().unwrap(); let gap = r.start_frame - m.end_frame();
            for g in 1..gap { m.freqs.push(prev + (f0 - prev) * g as f32 / gap as f32); }
            m.freqs.extend(r.freqs);
        } else { merged.push(r); }
    }
    merged.into_iter().filter(|r| r.freqs.len() >= min_len && is_doppler(r)).collect()
}

fn find_ridges_raw(st: &Stft, hum_base: f32) -> Vec<Ridge> {
    let hz = st.bin_hz();
    let lo = (BAND_LO / hz).ceil() as usize; let hi = ((BAND_HI / hz) as usize).min(st.n / 2 - 1);
    let mut active: Vec<(Ridge, usize)> = vec![]; // ridge, last frame seen
    let mut done = vec![];
    for f in 0..st.frames.len() {
        let m: Vec<f32> = (0..=st.n / 2).map(|k| st.mag(f, k) + 1e-9).collect();
        let mut band: Vec<f32> = m[lo..hi].to_vec();
        band.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = band[band.len() / 2];
        let mut peaks = vec![];
        for k in lo.max(1)..hi {
            if m[k] > m[k - 1] && m[k] >= m[k + 1] && m[k] > 8.0 * med {
                let fr = interp(&m, k, hz);
                if !near_hum(fr, hum_base, 1.2 * hz) { peaks.push(fr); }
            }
        }
        let mut used = vec![false; peaks.len()];
        for (r, last) in active.iter_mut() {
            let prev = *r.freqs.last().unwrap();
            let n = r.freqs.len(); let back = n.min(8);
            let slope = if back > 1 { (prev - r.freqs[n - back]) / (back - 1) as f32 } else { 0.0 };
            let pred = prev + slope * (f - *last) as f32;
            let mut best: Option<(usize, f32)> = None;
            for (i, p) in peaks.iter().enumerate() {
                let d = (p - pred).abs();
                if !used[i] && d < 2.5 * hz && best.is_none_or(|b| d < b.1) { best = Some((i, d)); }
            }
            if let Some((i, _)) = best {
                used[i] = true;
                // fill small gaps by holding frequency
                let gap = f - r.end_frame();
                for g in 1..gap { r.freqs.push(prev + (peaks[i] - prev) * g as f32 / gap as f32); }
                r.freqs.push(peaks[i]); *last = f;
            }
        }
        let (keep, ended): (Vec<_>, Vec<_>) = active.into_iter().partition(|(_, l)| f - *l <= 8);
        active = keep; done.extend(ended.into_iter().map(|(r, _)| r));
        for (i, p) in peaks.iter().enumerate() { if !used[i] { active.push((Ridge { start_frame: f, freqs: vec![*p] }, f)); } }
    }
    done.extend(active.into_iter().map(|(r, _)| r));
    done
}

/// A Doppler pass: overall descending, monotone-ish, drop of a few percent.
fn is_doppler(r: &Ridge) -> bool {
    let n = r.freqs.len(); let q = (n / 8).max(1);
    let head: f32 = r.freqs[..q].iter().sum::<f32>() / q as f32;
    let tail: f32 = r.freqs[n - q..].iter().sum::<f32>() / q as f32;
    let mean: f32 = r.freqs.iter().sum::<f32>() / n as f32;
    let ups: f32 = r.freqs.windows(2).map(|w| (w[1] - w[0]).max(0.0)).sum();
    let drop = head - tail;
    drop > 0.04 * mean && ups < 0.5 * drop
}

pub fn detect_hum(st: &Stft) -> Hum {
    let hz = st.bin_hz(); let nb = st.n / 2 + 1;
    // Per-bin median power over time: robust to transient passes.
    let step = (st.frames.len() / 300).max(1);
    let idx: Vec<usize> = (0..st.frames.len()).step_by(step).collect();
    let avg: Vec<f32> = (0..nb).map(|k| { let mut v: Vec<f32> = idx.iter().map(|&f| st.mag(f, k).powi(2)).collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2] }).collect();
    let mut best = Hum::default();
    for base in [50.0f32, 60.0] {
        let mut hs = vec![];
        for h in 1..=8 {
            let fh = base * h as f32; let k = (fh / hz).round() as usize;
            if k + 10 >= nb || k < 10 { break; }
            let pk = avg[k - 1].max(avg[k]).max(avg[k + 1]);
            let mut nbr: Vec<f32> = (3..10).flat_map(|d| [avg[k - d], avg[k + d]]).collect();
            nbr.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let ratio = pk / (nbr[nbr.len() / 2] + 1e-12);
            if ratio > 5.0 { hs.push(fh); }
        }
        if hs.len() > best.harmonics.len() { let has_low = hs.iter().any(|h| *h <= base * 2.0 + 1.0);
            best = Hum { detected: hs.len() >= 2 && has_low, base_hz: base, harmonics: hs }; }
    }
    best
}

fn group(st: &Stft, mut ridges: Vec<Ridge>) -> Vec<Flyover> {
    // Peak (closest-approach) time = when the ridge crosses its mid frequency.
    let mid_t = |r: &Ridge| {
        let (a, b) = (r.freqs[0], *r.freqs.last().unwrap()); let m = 0.5 * (a + b);
        let i = r.freqs.iter().position(|f| *f <= m).unwrap_or(0);
        st.frame_time(r.start_frame + i)
    };
    ridges.sort_by(|a, b| mid_t(a).partial_cmp(&mid_t(b)).unwrap());
    let mut out: Vec<Flyover> = vec![];
    for r in ridges {
        let t = mid_t(&r);
        let (s, e) = (st.frame_time(r.start_frame), st.frame_time(r.end_frame()));
        if let Some(fo) = out.last_mut() { if (fo.peak_s - t).abs() < 6.0 {
            fo.start_s = fo.start_s.min(s); fo.end_s = fo.end_s.max(e); fo.ridges.push(r); continue; } }
        out.push(Flyover { start_s: s, end_s: e, peak_s: t, f_start: r.freqs[0], f_end: *r.freqs.last().unwrap(), ridges: vec![r] });
    }
    out
}

pub fn analyze_stft(st: &Stft) -> Report {
    let hum = detect_hum(st);
    let flyovers = group(st, find_ridges(st, if hum.detected { hum.base_hz } else { 0.0 }));
    let n = flyovers.len() as u32;
    let mut score = (n * 30).min(60);
    let mut notes = vec![];
    if n > 0 { notes.push(format!("{} aircraft-like Doppler pass(es): their timing can be matched against public flight records.", n)); }
    if n >= 2 { score += 15; notes.push("Multiple passes: the gaps between them make the timing more distinctive.".into()); }
    if hum.detected { score += 15 + (hum.harmonics.len() as u32).min(10); notes.push(format!("{} Hz mains hum with {} harmonic(s): hints at region and can help date a recording.", hum.base_hz, hum.harmonics.len())); }
    let score = score.min(100);
    let level = Level::from_score(score);
    let narrowness = if n >= 2 { Narrowness::High } else if n == 1 || hum.detected { Narrowness::Moderate } else { Narrowness::Low };
    if notes.is_empty() { notes.push("No Doppler passes or mains hum found. This reduces risk but does not guarantee the clip is clean.".into()); }
    Report { duration_s: st.len as f32 / st.sr, flyovers, hum, score, level, narrowness, notes }
}

pub fn analyze(x: &[f32], sr: f32) -> Report { analyze_stft(&Stft::new(x, sr)) }
