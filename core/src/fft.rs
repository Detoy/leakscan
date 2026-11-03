//! Minimal in-place radix-2 complex FFT (no dependencies).
#[derive(Clone, Copy, Debug, Default)]
pub struct C { pub re: f32, pub im: f32 }

pub fn fft(buf: &mut [C], inverse: bool) {
    let n = buf.len();
    assert!(n.is_power_of_two());
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 { j ^= bit; bit >>= 1; }
        j |= bit;
        if i < j { buf.swap(i, j); }
    }
    let mut len = 2;
    while len <= n {
        let ang = std::f64::consts::TAU / len as f64 * if inverse { 1.0 } else { -1.0 };
        let (wr, wi) = (ang.cos(), ang.sin());
        for start in (0..n).step_by(len) {
            let (mut cr, mut ci) = (1.0f64, 0.0f64);
            for k in 0..len / 2 {
                let a = buf[start + k];
                let b = buf[start + k + len / 2];
                let (tr, ti) = (b.re as f64 * cr - b.im as f64 * ci, b.re as f64 * ci + b.im as f64 * cr);
                buf[start + k] = C { re: (a.re as f64 + tr) as f32, im: (a.im as f64 + ti) as f32 };
                buf[start + k + len / 2] = C { re: (a.re as f64 - tr) as f32, im: (a.im as f64 - ti) as f32 };
                let ncr = cr * wr - ci * wi; ci = cr * wi + ci * wr; cr = ncr;
            }
        }
        len <<= 1;
    }
    if inverse { let s = 1.0 / n as f32; for c in buf.iter_mut() { c.re *= s; c.im *= s; } }
}
