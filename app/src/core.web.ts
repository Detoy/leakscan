// Web implementation: Rust core compiled to WASM (wasm-pack, no-modules target),
// loaded from this site's own /core folder. Everything runs locally in the browser.
import type { Audio, DbScale, Report, Spectrogram } from './types';

export const supported = true;
const MAX_HZ = 600;
let W: any = null;

function base(): string {
  const p = window.location.pathname;
  return p.startsWith('/leakscan') ? '/leakscan' : '';
}

export async function loadCore(): Promise<void> {
  if (W) return;
  await new Promise<void>((res, rej) => {
    const s = document.createElement('script');
    s.src = `${base()}/core/leakscan_core.js`; s.onload = () => res(); s.onerror = () => rej(new Error('core script failed'));
    document.head.appendChild(s);
  });
  // no-modules glue declares a top-level `let`, which is not a window property; expose it.
  const expose = document.createElement('script');
  expose.textContent = 'window.__leakscanCore = wasm_bindgen;';
  document.head.appendChild(expose);
  const wb = (window as any).__leakscanCore;
  await wb({ module_or_path: `${base()}/core/leakscan_core_bg.wasm` });
  W = wb;
}

export function pickAudio(): Promise<Audio> {
  return new Promise((resolve, reject) => {
    const input = document.createElement('input');
    input.type = 'file'; input.accept = 'audio/*,video/*';
    input.onchange = async () => {
      const f = input.files?.[0]; if (!f) return reject(new Error('No file'));
      try {
        const buf = await f.arrayBuffer();
        const ctx = new OfflineAudioContext(1, 1, 44100); // decodes + resamples locally
        const ab = await ctx.decodeAudioData(buf);
        const mono = new Float32Array(ab.length);
        for (let c = 0; c < ab.numberOfChannels; c++) { const d = ab.getChannelData(c); for (let i = 0; i < d.length; i++) mono[i] += d[i] / ab.numberOfChannels; }
        resolve({ samples: mono, sr: ab.sampleRate, name: f.name });
      } catch (e) { reject(e); }
    };
    input.click();
  });
}

export function demoAudio(): Audio { const sr = 16000; return { samples: W.demo(sr), sr, name: 'synthetic-demo.wav' }; }
export function analyze(a: Audio): Report { return JSON.parse(W.analyze(a.samples, a.sr)); }
export function scrub(a: Audio, noiseDbfs = -52): Audio {
  return { samples: W.scrub(a.samples, a.sr, noiseDbfs), sr: a.sr, name: a.name.replace(/\.[^.]+$/, '') + '-scrubbed.wav' };
}

function color(v: number): [number, number, number] {
  const t = v / 255; // dark navy -> violet -> amber
  return [Math.min(255, 20 + 400 * t * t), Math.min(255, 10 + 180 * t ** 3 + 30 * t), Math.min(255, 40 + 200 * t * (1 - t) * 1.6)];
}

const HEADER = 6;

/** Renders `a` with detections from `r`. Pass the scale of an earlier render to keep brightness comparable. */
export function spectrogram(a: Audio, r: Report, scale?: DbScale): Spectrogram {
  const out: Float32Array = W.spectrogram(a.samples, a.sr, MAX_HZ, scale?.lo, scale?.hi);
  const frames = out[0], bins = out[1], binHz = out[3];
  const c = document.createElement('canvas'); c.width = frames; c.height = bins;
  const g = c.getContext('2d')!; const img = g.createImageData(frames, bins);
  for (let f = 0; f < frames; f++) for (let k = 0; k < bins; k++) {
    const [R, G, B] = color(out[HEADER + f * bins + k]); const p = ((bins - 1 - k) * frames + f) * 4;
    img.data[p] = R; img.data[p + 1] = G; img.data[p + 2] = B; img.data[p + 3] = 255;
  }
  g.putImageData(img, 0, 0);
  const big = document.createElement('canvas'); big.width = 1200; big.height = 360;
  const G2 = big.getContext('2d')!; G2.imageSmoothingEnabled = true; G2.drawImage(c, 0, 0, big.width, big.height);
  const sx = big.width / frames, sy = big.height / bins;
  if (r.hum.detected) {
    G2.setLineDash([6, 6]); G2.strokeStyle = 'rgba(56,189,248,0.9)'; G2.lineWidth = 1.5;
    for (let h = r.hum.base_hz; h < MAX_HZ; h += r.hum.base_hz) { const y = big.height - (h / binHz) * sy; G2.beginPath(); G2.moveTo(0, y); G2.lineTo(big.width, y); G2.stroke(); }
    G2.setLineDash([]);
  }
  G2.strokeStyle = '#ff7a45'; G2.lineWidth = 3; G2.shadowColor = '#ff7a45'; G2.shadowBlur = 8;
  for (const fo of r.flyovers) for (const rg of fo.ridges) {
    G2.beginPath();
    rg.freqs.forEach((fr, i) => { const x = (rg.start_frame + i) * sx, y = big.height - (fr / binHz) * sy; i ? G2.lineTo(x, y) : G2.moveTo(x, y); });
    G2.stroke();
  }
  G2.shadowBlur = 0;
  G2.fillStyle = 'rgba(255,255,255,0.6)'; G2.font = '12px sans-serif';
  for (let hz = 100; hz < MAX_HZ; hz += 100) G2.fillText(`${hz} Hz`, 6, big.height - (hz / binHz) * sy - 2);
  return { url: big.toDataURL('image/png'), scale: { lo: out[4], hi: out[5] } };
}

/** Minimal 16-bit PCM WAV: only RIFF/fmt/data chunks, so no metadata survives. */
function wavBlob(a: Audio): Blob {
  const n = a.samples.length, b = new ArrayBuffer(44 + n * 2), v = new DataView(b);
  const w = (o: number, s: string) => [...s].forEach((ch, i) => v.setUint8(o + i, ch.charCodeAt(0)));
  w(0, 'RIFF'); v.setUint32(4, 36 + n * 2, true); w(8, 'WAVEfmt '); v.setUint32(16, 16, true); v.setUint16(20, 1, true); v.setUint16(22, 1, true);
  v.setUint32(24, a.sr, true); v.setUint32(28, a.sr * 2, true); v.setUint16(32, 2, true); v.setUint16(34, 16, true); w(36, 'data'); v.setUint32(40, n * 2, true);
  for (let i = 0; i < n; i++) v.setInt16(44 + i * 2, Math.max(-1, Math.min(1, a.samples[i])) * 32767, true);
  return new Blob([b], { type: 'audio/wav' });
}
export function audioUrl(a: Audio): string { return URL.createObjectURL(wavBlob(a)); }
export function exportWav(a: Audio): void {
  const link = document.createElement('a'); link.href = audioUrl(a); link.download = a.name; link.click();
}
