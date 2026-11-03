use crate::stft::Stft;
/// Log-magnitude spectrogram up to `max_hz`, normalised to 0–255.
/// Layout: [frames, bins, hop_s, bin_hz, data...] (frame-major).
pub fn spectrogram(x: &[f32], sr: f32, max_hz: f32) -> Vec<f32> {
    let st = Stft::new(x, sr);
    let bins = ((max_hz / st.bin_hz()) as usize).min(st.n / 2);
    let mut db: Vec<f32> = Vec::with_capacity(st.frames.len() * bins);
    for f in 0..st.frames.len() { for k in 0..bins { db.push(20.0 * (st.mag(f, k) + 1e-6).log10()); } }
    let mut s = db.clone(); s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let (lo, hi) = (s[s.len() / 2], s[s.len() - 1 - s.len() / 2000]);
    let mut out = vec![st.frames.len() as f32, bins as f32, st.hop as f32 / sr, st.bin_hz()];
    out.extend(db.iter().map(|v| ((v - lo) / (hi - lo + 1e-6)).clamp(0.0, 1.0) * 255.0));
    out
}
