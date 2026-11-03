//! Scrubbing: spectral notches along detected ridges and hum harmonics, plus light noise.
use crate::{detect::{analyze_stft, Report}, stft::Stft, synth::white_noise};

pub struct ScrubOptions { pub noise_dbfs: f32, pub notch_bins: usize, pub seed: u32, pub passes: usize }
impl Default for ScrubOptions { fn default() -> Self { Self { noise_dbfs: -52.0, notch_bins: 3, seed: 7, passes: 3 } } }

/// What a scrub did, summed over every detect-and-notch pass.
pub struct Scrubbed { pub audio: Vec<f32>, pub before: Report, pub ridges_notched: usize, pub hum_removed_hz: Option<f32> }

pub struct Suggestion { pub start_s: f32, pub end_s: f32, pub reason: String }

/// Notch attenuation (-40 dB).
const NOTCH_GAIN: f32 = 0.01;
/// Frames notched past each ridge end, to catch faint tails the tracker dropped.
const RIDGE_PAD: usize = 6;
const HUM_MAX_HZ: f32 = 1000.0;

pub fn scrub_with(x: &[f32], sr: f32, opt: &ScrubOptions) -> Scrubbed {
    let mut st = Stft::new(x, sr);
    let before = analyze_stft(&st);
    let (hz, bins, w) = (st.bin_hz(), st.n / 2 + 1, opt.notch_bins as isize);
    let damp = |st: &mut Stft, f: usize, freq: f32| {
        let k = (freq / hz).round() as isize;
        for b in (k - w..=k + w).filter(|&b| b > 0 && (b as usize) < bins) {
            let c = &mut st.frames[f][b as usize];
            c.re *= NOTCH_GAIN;
            c.im *= NOTCH_GAIN;
        }
    };
    let (mut ridges_notched, mut hum_removed_hz) = (0, None);
    let mut rep = before.clone();
    for pass in 0..opt.passes {
        if pass > 0 { rep = analyze_stft(&st); }
        if rep.flyovers.is_empty() && !rep.hum.detected { break; }
        let nf = st.frames.len();
        for r in rep.flyovers.iter().flat_map(|fo| &fo.ridges) {
            ridges_notched += 1;
            let (f0, f1) = (r.freqs[0], *r.freqs.last().unwrap());
            for f in r.start_frame.saturating_sub(RIDGE_PAD)..(r.end_frame() + RIDGE_PAD + 1).min(nf) {
                let freq = if f < r.start_frame { f0 } else if f > r.end_frame() { f1 } else { r.freqs[f - r.start_frame] };
                damp(&mut st, f, freq);
            }
        }
        if rep.hum.detected {
            let base = rep.hum.base_hz;
            hum_removed_hz = Some(base);
            for h in (1..).map(|k| k as f32 * base).take_while(|&h| h < HUM_MAX_HZ) {
                for f in 0..nf { damp(&mut st, f, h); }
            }
        }
    }
    let mut audio = st.inverse();
    let amp = 10f32.powf(opt.noise_dbfs / 20.0) * 3f32.sqrt();
    let mut noise = white_noise(opt.seed);
    for v in audio.iter_mut() { *v += noise() * amp; }
    Scrubbed { audio, before, ridges_notched, hum_removed_hz }
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
