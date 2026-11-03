use crate::stft::Stft;

/// dB range mapped to pixel 0..=255. Reusing one scale across clips keeps their brightness comparable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DbScale { pub lo: f32, pub hi: f32 }

pub struct Spectrogram { pub frames: usize, pub bins: usize, pub hop_s: f32, pub bin_hz: f32, pub scale: DbScale, pub pixels: Vec<u8> }

/// Log-magnitude spectrogram up to `max_hz`, frame-major. With `scale: None` the range is fitted to this clip
/// (median to 99.95th percentile); pass the "before" scale when drawing the "after" clip.
pub fn spectrogram(x: &[f32], sr: f32, max_hz: f32, scale: Option<DbScale>) -> Spectrogram {
    let st = Stft::new(x, sr);
    let bins = ((max_hz / st.bin_hz()) as usize).min(st.n / 2);
    let db: Vec<f32> = (0..st.frames.len()).flat_map(|f| (0..bins).map(move |k| (f, k)))
        .map(|(f, k)| 20.0 * (st.mag(f, k) + 1e-6).log10()).collect();
    let scale = scale.unwrap_or_else(|| {
        let mut s = db.clone(); s.sort_by(f32::total_cmp);
        DbScale { lo: s[s.len() / 2], hi: s[s.len() - 1 - s.len() / 2000] }
    });
    let pixels = db.iter().map(|v| (((v - scale.lo) / (scale.hi - scale.lo + 1e-6)).clamp(0.0, 1.0) * 255.0) as u8).collect();
    Spectrogram { frames: st.frames.len(), bins, hop_s: st.hop as f32 / sr, bin_hz: st.bin_hz(), scale, pixels }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{scrub, synth};
    const SR: f32 = 16000.0;
    const MAX_HZ: f32 = 600.0;
    fn mean(p: &[u8]) -> f32 { p.iter().map(|&v| v as f32).sum::<f32>() / p.len() as f32 }

    #[test] fn after_reuses_before_scale_and_reads_quieter() {
        let x = synth::demo(SR);
        let y = scrub::scrub(&x, SR);
        let before = spectrogram(&x, SR, MAX_HZ, None);
        let after = spectrogram(&y, SR, MAX_HZ, Some(before.scale));
        assert_eq!(after.scale, before.scale);
        assert!(mean(&after.pixels) < mean(&before.pixels), "{} !< {}", mean(&after.pixels), mean(&before.pixels));
    }

    #[test] fn self_scaling_hides_the_reduction() {
        let x = synth::demo(SR);
        let y = scrub::scrub(&x, SR);
        let before = spectrogram(&x, SR, MAX_HZ, None);
        let shared = spectrogram(&y, SR, MAX_HZ, Some(before.scale));
        let own = spectrogram(&y, SR, MAX_HZ, None);
        assert!(own.scale.hi < before.scale.hi);
        assert!(mean(&own.pixels) > mean(&shared.pixels));
    }
}
