//! Writes the synthetic demo clip to a WAV file: cargo run --example gen_demo -- out.wav
fn main() {
    let path = std::env::args().nth(1).unwrap_or("demo.wav".into());
    let sr = 16000u32; let x = leakscan_core::synth::demo(sr as f32);
    let mut b = Vec::new(); let d = (x.len() * 2) as u32;
    b.extend(b"RIFF"); b.extend((36 + d).to_le_bytes()); b.extend(b"WAVEfmt ");
    b.extend(16u32.to_le_bytes()); b.extend(1u16.to_le_bytes()); b.extend(1u16.to_le_bytes());
    b.extend(sr.to_le_bytes()); b.extend((sr * 2).to_le_bytes()); b.extend(2u16.to_le_bytes()); b.extend(16u16.to_le_bytes());
    b.extend(b"data"); b.extend(d.to_le_bytes());
    for v in x { b.extend(((v.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes()); }
    std::fs::write(&path, b).unwrap(); println!("wrote {path}");
}
