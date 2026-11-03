//! LeakScan core: on-device detection and scrubbing of audio cues (aircraft Doppler
//! passes, mains hum) that could help someone work out where/when a clip was recorded.
//! Defensive only: no geolocation, no flight-data matching, no coordinate output.
//! Scrubbing reduces these cues; it does not guarantee a clip is safe.
pub mod fft; pub mod stft; pub mod detect; pub mod scrub; pub mod synth; pub mod spectro;

use detect::Report;

fn esc(s: &str) -> String { s.replace('\\', "\\\\").replace('"', "\\\"") }

pub fn report_json(r: &Report) -> String {
    let fl: Vec<String> = r.flyovers.iter().map(|f| {
        let ridges: Vec<String> = f.ridges.iter().map(|rg| format!("{{\"start_frame\":{},\"freqs\":[{}]}}", rg.start_frame,
            rg.freqs.iter().map(|x| format!("{:.1}", x)).collect::<Vec<_>>().join(","))).collect();
        format!("{{\"start_s\":{:.2},\"end_s\":{:.2},\"peak_s\":{:.2},\"f_start\":{:.1},\"f_end\":{:.1},\"ridges\":[{}]}}",
            f.start_s, f.end_s, f.peak_s, f.f_start, f.f_end, ridges.join(","))
    }).collect();
    let cuts: Vec<String> = scrub::suggest_cuts(r).iter().map(|c| format!("{{\"start_s\":{:.2},\"end_s\":{:.2},\"reason\":\"{}\"}}", c.start_s, c.end_s, esc(&c.reason))).collect();
    format!("{{\"duration_s\":{:.2},\"score\":{},\"level\":\"{}\",\"narrowness\":\"{}\",\"hum\":{{\"detected\":{},\"base_hz\":{},\"harmonics\":[{}]}},\"flyovers\":[{}],\"cuts\":[{}],\"notes\":[{}]}}",
        r.duration_s, r.score, r.level, r.narrowness, r.hum.detected, r.hum.base_hz,
        r.hum.harmonics.iter().map(|h| format!("{}", h)).collect::<Vec<_>>().join(","), fl.join(","), cuts.join(","),
        r.notes.iter().map(|n| format!("\"{}\"", esc(n))).collect::<Vec<_>>().join(","))
}

#[cfg(feature = "wasm")]
mod wasm {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen] pub fn analyze(x: &[f32], sr: f32) -> String { super::report_json(&super::detect::analyze(x, sr)) }
    #[wasm_bindgen] pub fn scrub(x: &[f32], sr: f32, noise_dbfs: f32) -> Vec<f32> {
        super::scrub::scrub_with(x, sr, &super::scrub::ScrubOptions { noise_dbfs, ..Default::default() }).0 }
    #[wasm_bindgen] pub fn demo(sr: f32) -> Vec<f32> { super::synth::demo(sr) }
    /// Returns [frames, bins, hop_s, bin_hz, ...u8 dB image row-major by frame].
    #[wasm_bindgen] pub fn spectrogram(x: &[f32], sr: f32, max_hz: f32) -> Vec<f32> { super::spectro::spectrogram(x, sr, max_hz) }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SR: f32 = 16000.0;

    #[test] fn fft_roundtrip() {
        let x: Vec<f32> = (0..20000).map(|i| (i as f32 * 0.01).sin()).collect();
        let y = stft::Stft::new(&x, SR).inverse();
        let err = x.iter().zip(&y).skip(5000).take(10000).map(|(a, b)| (a - b).abs()).fold(0.0, f32::max);
        assert!(err < 1e-3, "err {err}");
    }
    #[test] fn detects_demo_passes_and_hum() {
        let r = detect::analyze(&synth::demo(SR), SR);
        assert_eq!(r.flyovers.len(), 2, "{:?}", r.flyovers.iter().map(|f| f.peak_s).collect::<Vec<_>>());
        assert!((r.flyovers[0].peak_s - 17.0).abs() < 2.0);
        assert!((r.flyovers[1].peak_s - 44.0).abs() < 2.0);
        assert!(r.hum.detected && r.hum.base_hz == 50.0);
        assert_eq!(r.level, "high");
    }
    #[test] fn detects_60hz() {
        let x = synth::synth(SR, 20.0, &[], Some(60.0), 3);
        let r = detect::analyze(&x, SR);
        assert!(r.hum.detected && r.hum.base_hz == 60.0); assert!(r.flyovers.is_empty());
    }
    #[test] fn clean_noise_is_low() {
        let r = detect::analyze(&synth::synth(SR, 20.0, &[], None, 9), SR);
        assert!(r.flyovers.is_empty() && !r.hum.detected && r.score == 0);
    }
    #[test] fn flat_tone_is_not_a_flyover() {
        let mut x = synth::synth(SR, 20.0, &[], None, 5);
        for (i, v) in x.iter_mut().enumerate() { *v += 0.05 * (2.0 * 3.14159 * 333.0 * i as f32 / SR).sin(); }
        assert!(detect::analyze(&x, SR).flyovers.is_empty());
    }
    #[test] fn scrub_lowers_score() {
        let x = synth::demo(SR);
        let (y, before) = scrub::scrub_with(&x, SR, &Default::default());
        let after = detect::analyze(&y, SR);
        assert!(after.score < before.score, "{} -> {}", before.score, after.score);
        assert!(after.flyovers.is_empty() && !after.hum.detected, "{:?} {:?}", after.flyovers.iter().map(|f| (f.peak_s,f.f_start,f.f_end)).collect::<Vec<_>>(), after.hum);
    }
    #[test] fn json_ok() { let j = report_json(&detect::analyze(&synth::demo(SR), SR)); assert!(j.starts_with('{') && j.contains("\"flyovers\"")); }
    #[test] fn works_at_48k() { assert_eq!(detect::analyze(&synth::demo(48000.0), 48000.0).flyovers.len(), 2); }
}
