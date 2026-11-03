export type Ridge = { start_frame: number; freqs: number[] };
export type Flyover = { start_s: number; end_s: number; peak_s: number; f_start: number; f_end: number; ridges: Ridge[] };
export type Report = {
  duration_s: number; score: number; level: 'low' | 'medium' | 'high'; narrowness: 'low' | 'moderate' | 'high';
  hum: { detected: boolean; base_hz: number; harmonics: number[] };
  flyovers: Flyover[]; cuts: { start_s: number; end_s: number; reason: string }[]; notes: string[];
};
export type Audio = { samples: Float32Array; sr: number; name: string };
export type DbScale = { lo: number; hi: number };
export type Spectrogram = { url: string; scale: DbScale };
/** What a scrub did, summed over every detect-and-notch pass. */
export type Scrubbed = { audio: Audio; ridgesNotched: number; humRemovedHz: number | null };
