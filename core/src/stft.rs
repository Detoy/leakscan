//! Short-time Fourier transform with Hann window, hop = n/4 (perfect OLA reconstruction).
use crate::fft::{fft, C};

pub struct Stft { pub n: usize, pub hop: usize, pub sr: f32, pub frames: Vec<Vec<C>>, pub len: usize }

/// FFT size giving ~3–4 Hz bins at any sample rate (needed to resolve slow Doppler sweeps).
fn fft_size(sr: f32) -> usize { ((sr / 4.0) as usize).next_power_of_two().max(256) }

fn hann(n: usize) -> Vec<f32> {
    (0..n).map(|i| 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / n as f32).cos()).collect()
}

impl Stft {
    pub fn new(x: &[f32], sr: f32) -> Self {
        let n = fft_size(sr); let hop = n / 4; let w = hann(n);
        let nf = if x.len() <= n { 1 } else { (x.len() - n) / hop + 2 };
        let mut frames = Vec::with_capacity(nf);
        for f in 0..nf {
            let s = f * hop;
            let mut b: Vec<C> = (0..n).map(|i| C { re: x.get(s + i).copied().unwrap_or(0.0) * w[i], im: 0.0 }).collect();
            fft(&mut b, false);
            b.truncate(n / 2 + 1);
            frames.push(b);
        }
        Stft { n, hop, sr, frames, len: x.len() }
    }
    pub fn bin_hz(&self) -> f32 { self.sr / self.n as f32 }
    pub fn frame_time(&self, f: usize) -> f32 { (f * self.hop + self.n / 2) as f32 / self.sr }
    pub fn mag(&self, f: usize, k: usize) -> f32 { let c = self.frames[f][k]; (c.re * c.re + c.im * c.im).sqrt() }

    /// Inverse STFT via weighted overlap-add.
    pub fn inverse(&self) -> Vec<f32> {
        let n = self.n; let w = hann(n);
        let mut out = vec![0.0f32; self.len + n]; let mut norm = vec![0.0f32; self.len + n];
        for (f, fr) in self.frames.iter().enumerate() {
            let mut b = vec![C::default(); n];
            for k in 0..=n / 2 { b[k] = fr[k]; if k > 0 && k < n / 2 { b[n - k] = C { re: fr[k].re, im: -fr[k].im }; } }
            fft(&mut b, true);
            let s = f * self.hop;
            for i in 0..n { out[s + i] += b[i].re * w[i]; norm[s + i] += w[i] * w[i]; }
        }
        out.truncate(self.len);
        out.iter().zip(norm.iter()).map(|(o, z)| if *z > 1e-3 { o / z } else { *o }).collect()
    }
}
