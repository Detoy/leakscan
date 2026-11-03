// Native (iOS/Android) placeholder. The Rust core runs on web via WASM today;
// native bindings (uniffi / JSI) are on the roadmap. No network is ever used.
import type { Audio, DbScale, Report, Scrubbed, Spectrogram } from './types';
export const supported = false;
const nope = () => { throw new Error('The on-device core is not wired up on native yet. Use the web build for now.'); };
export async function loadCore(): Promise<void> { nope(); }
export async function pickAudio(): Promise<Audio> { return nope(); }
export function demoAudio(): Audio { return nope(); }
export function analyze(_a: Audio): Report { return nope(); }
export function scrub(_a: Audio): Scrubbed { return nope(); }
export function spectrogram(_a: Audio, _r: Report, _scale?: DbScale): Spectrogram { return nope(); }
export function exportWav(_a: Audio): void { nope(); }
export function audioUrl(_a: Audio): string { return nope(); }
