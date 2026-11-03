//! Scrubbing: spectral notches along detected ridges and hum harmonics, plus light noise.
use crate::{detect::{analyze_stft, Report}, stft::Stft};

pub struct ScrubOptions { pub noise_dbfs: f32, pub notch_bins: usize, pub seed: u32, pub passes: usize }
impl Default for ScrubOptions { fn default() -> Self { Self { noise_dbfs: -52.0, notch_bins: 3, seed: 7, passes: 3 } } }

/// What a scrub did, summed over every detect-and-notch pass.
pub struct Scrubbed { pub audio: Vec<f32>, pub before: Report, pub ridges_notched: usize, pub hum_removed_hz: Option<f32> }

pub struct Suggestion { pub start_s: f32, pub end_s: f32, pub reason: String }

pub fn scrub_with(x: &[f32], sr: f32, opt: &ScrubOptions) -> Scrubbed {
    let mut st = Stft::new(x, sr);
    let first = analyze_stft(&st);
    let mut rep = first.clone();
    let hz = st.bin_hz(); let nb = st.n / 2 + 1; let w = opt.notch_bins as isize;
    let att = 0.01f32;
    let (mut ridges_notched, mut hum_removed_hz) = (0, None);
    let damp = |st: &mut Stft, f: usize, freq: f32| {
        let k = (freq / hz).round() as isize;
        for d in -w..=w { let b = k + d; if b > 0 && (b as usize) < nb { let c = &mut st.frames[f][b as usize]; c.re *= att; c.im *= att; } }
    };
    for pass in 0..opt.passes {
    if pass > 0 { rep = analyze_stft(&st); if rep.flyovers.is_empty() && !rep.hum.detected { break; } }
    let pad = 6usize; // extend notch beyond the detected ends (faint tails)
    for fo in &rep.flyovers { for r in &fo.ridges {
        ridges_notched += 1;
        let nf = st.frames.len();
        let (f0, f1) = (r.freqs[0], *r.freqs.last().unwrap());
        for f in r.start_frame.saturating_sub(pad)..(r.end_frame() + pad + 1).min(nf) {
            let fr = if f < r.start_frame { f0 } else if f > r.end_frame() { f1 } else { r.freqs[f - r.start_frame] };
            damp(&mut st, f, fr);
        }
    } }
    if rep.hum.detected {
        hum_removed_hz = Some(rep.hum.base_hz);
        let mut h = rep.hum.base_hz;
        while h < 1000.0 { for f in 0..st.frames.len() { damp(&mut st, f, h); } h += rep.hum.base_hz; }
    }
    }
    let mut y = st.inverse();
    let amp = 10f32.powf(opt.noise_dbfs / 20.0) * 3f32.sqrt();
    let mut s = opt.seed.max(1);
    for v in y.iter_mut() { s ^= s << 13; s ^= s >> 17; s ^= s << 5; *v += ((s as f32 / u32::MAX as f32) * 2.0 - 1.0) * amp; }
    Scrubbed { audio: y, before: first, ridges_notched, hum_removed_hz }
}

pub fn scrub(x: &[f32], sr: f32) -> Vec<f32> { scrub_with(x, sr, &ScrubOptions::default()).audio }

/// Suggested cuts: remove each pass's closest-approach window (the most identifying part),
/// which also breaks the timing gap between passes.
pub fn suggest_cuts(rep: &Report) -> Vec<Suggestion> {
    rep.flyovers.iter().map(|f| Suggestion {
        start_s: (f.peak_s - 3.0).max(0.0), end_s: (f.peak_s + 3.0).min(rep.duration_s),
        reason: "Closest-approach part of a Doppler pass; cutting it also breaks the gap between passes.".into(),
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{detect::analyze, synth};
    const SR: f32 = 16000.0;

    #[test] fn counts_ridges_from_every_pass() {
        let x = synth::demo(SR);
        let out = scrub_with(&x, SR, &Default::default());
        let first_pass: usize = out.before.flyovers.iter().map(|f| f.ridges.len()).sum();
        let single = scrub_with(&x, SR, &ScrubOptions { passes: 1, ..Default::default() });
        assert_eq!(single.ridges_notched, first_pass);
        assert!(out.ridges_notched > first_pass, "{} !> {}", out.ridges_notched, first_pass);
        assert_eq!(out.hum_removed_hz, Some(50.0));
        assert!(analyze(&out.audio, SR).flyovers.is_empty());
    }
}
