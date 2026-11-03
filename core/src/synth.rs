//! Synthetic demo signals: aircraft-like Doppler passes + mains hum over background noise.
//! All demo audio in this project comes from here; no real recordings are used.

/// Deterministic uniform noise in [-1, 1] (xorshift32).
pub(crate) fn white_noise(seed: u32) -> impl FnMut() -> f32 {
    let mut s = seed.max(1);
    move || { s ^= s << 13; s ^= s >> 17; s ^= s << 5; (s as f32 / u32::MAX as f32) * 2.0 - 1.0 }
}

pub struct Pass { pub t_closest: f32, pub f0: f32, pub speed: f32, pub dist: f32, pub amp: f32 }

pub fn synth(sr: f32, secs: f32, passes: &[Pass], hum_hz: Option<f32>, seed: u32) -> Vec<f32> {
    let n = (sr * secs) as usize; let c = 343.0f32;
    let mut x = vec![0.0f32; n];
    let mut noise = white_noise(seed); let mut brown = 0.0f32;
    for (i, v) in x.iter_mut().enumerate() {
        let wn = noise();
        brown = (1.0 - 80.0 / sr) * brown + (800.0 / sr) * wn;
        *v = 0.01 * wn + 0.01 * brown;
        if let Some(h) = hum_hz { let t = i as f64 / sr as f64;
            for k in 1..=4 { *v += 0.01 / k as f32 * (std::f64::consts::TAU * (h as f64 * k as f64 * t).fract()).sin() as f32; } }
    }
    for p in passes {
        let mut ph = [0.0f32; 2];
        for (i, v) in x.iter_mut().enumerate() {
            let t = i as f32 / sr - p.t_closest; let along = p.speed * t;
            let r = (p.dist * p.dist + along * along).sqrt();
            let vr = p.speed * along / r; // receding > 0
            let f = p.f0 * c / (c + vr);
            let env = p.amp * (p.dist / r).powf(1.5);
            for (h, phase) in ph.iter_mut().enumerate() {
                *phase += std::f32::consts::TAU * f * (h + 1) as f32 / sr;
                if *phase > std::f32::consts::TAU { *phase -= std::f32::consts::TAU; }
                *v += env / (h + 1) as f32 * phase.sin();
            }
        }
    }
    x
}

/// The standard demo clip: 60 s, two passes 30 s apart, 50 Hz hum.
pub fn demo(sr: f32) -> Vec<f32> {
    synth(sr, 60.0, &[
        Pass { t_closest: 17.0, f0: 115.0, speed: 55.0, dist: 300.0, amp: 0.06 },
        Pass { t_closest: 44.0, f0: 150.0, speed: 60.0, dist: 400.0, amp: 0.05 },
    ], Some(50.0), 42)
}
