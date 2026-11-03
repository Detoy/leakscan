import React, { useEffect, useState } from 'react';
import { ActivityIndicator, Image, Platform, Pressable, ScrollView, StyleSheet, Text, View } from 'react-native';
import { StatusBar } from 'expo-status-bar';
import * as core from './src/core';
import type { Audio, Report } from './src/types';

const C = { bg: '#0b0d1a', card: '#151933', line: '#262b4d', text: '#eef0ff', dim: '#9aa0c8', accent: '#ff7a45', ok: '#3ddc97', mid: '#ffcf5a', hi: '#ff5a6e', hum: '#38bdf8' };
const levelColor = (l: string) => (l === 'high' ? C.hi : l === 'medium' || l === 'moderate' ? C.mid : C.ok);
const fmt = (s: number) => `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, '0')}`;

type Stage = 'home' | 'results' | 'scrubbed';

function Btn({ label, onPress, kind = 'primary' }: { label: string; onPress: () => void; kind?: 'primary' | 'ghost' }) {
  return (
    <Pressable onPress={onPress} style={[s.btn, kind === 'ghost' && s.ghost]}>
      <Text style={[s.btnText, kind === 'ghost' && { color: C.text }]}>{label}</Text>
    </Pressable>
  );
}

function Score({ r, title }: { r: Report; title: string }) {
  return (
    <View style={[s.card, { flex: 1, minWidth: 240 }]}>
      <Text style={s.label}>{title}</Text>
      <Text style={[s.score, { color: levelColor(r.level) }]}>{r.score}<Text style={s.scoreMax}>/100</Text></Text>
      <Text style={s.body}>Exposure: <Text style={{ color: levelColor(r.level), fontWeight: '700' }}>{r.level.toUpperCase()}</Text></Text>
      <Text style={s.body}>Aircraft passes: <Text style={s.b}>{r.flyovers.length}</Text></Text>
      <Text style={s.body}>Mains hum: <Text style={s.b}>{r.hum.detected ? `${r.hum.base_hz} Hz + ${r.hum.harmonics.length - 1} harmonics` : 'none found'}</Text></Text>
      <Text style={s.body}>How narrow (relative): <Text style={{ color: levelColor(r.narrowness), fontWeight: '700' }}>{r.narrowness}</Text></Text>
    </View>
  );
}

function AudioEl({ a }: { a: Audio }) {
  if (Platform.OS !== 'web') return null;
  const [url] = useState(() => core.audioUrl(a));
  return React.createElement('audio', { src: url, controls: true, style: { width: '100%', marginTop: 8 } });
}

export default function App() {
  const [ready, setReady] = useState(false);
  const [busy, setBusy] = useState('');
  const [err, setErr] = useState('');
  const [stage, setStage] = useState<Stage>('home');
  const [orig, setOrig] = useState<{ a: Audio; r: Report; img: string } | null>(null);
  const [clean, setClean] = useState<{ a: Audio; r: Report; img: string } | null>(null);

  const run = (label: string, fn: () => Promise<void> | void) => {
    setBusy(label); setErr('');
    setTimeout(async () => { try { await fn(); } catch (e: any) { setErr(String(e?.message ?? e)); } finally { setBusy(''); } }, 30);
  };
  const scan = (a: Audio) => { const r = core.analyze(a); setOrig({ a, r, img: core.spectrogramUrl(a, r) }); setClean(null); setStage('results'); };
  const doScrub = () => { if (!orig) return; const a = core.scrub(orig.a); const r = core.analyze(a); setClean({ a, r, img: core.spectrogramUrl(a, r) }); setStage('scrubbed'); };

  useEffect(() => {
    if (!core.supported) return;
    core.loadCore().then(() => {
      setReady(true);
      const demo = Platform.OS === 'web' ? new URLSearchParams(window.location.search).get('demo') : null;
      if (demo) run('Scanning demo…', () => { const a = core.demoAudio(); scan(a);
        if (demo === 'scrub') { const r = core.analyze(a); const b = core.scrub(a); const rb = core.analyze(b);
          setOrig({ a, r, img: core.spectrogramUrl(a, r) }); setClean({ a: b, r: rb, img: core.spectrogramUrl(b, rb) }); setStage('scrubbed'); } });
    }).catch((e) => setErr(String(e)));
  }, []);

  return (
    <View style={s.root}>
      <StatusBar style="light" />
      <ScrollView contentContainerStyle={s.page}>
        <View style={s.header}>
          <Text style={s.logo}>Leak<Text style={{ color: C.accent }}>Scan</Text></Text>
          <Text style={s.pill}>on-device · no uploads · open source</Text>
        </View>

        {!core.supported && <View style={s.card}><Text style={s.body}>The native build doesn't have the on-device core wired in yet. Use the web version for now.</Text></View>}
        {!!err && <View style={[s.card, { borderColor: C.hi }]}><Text style={[s.body, { color: C.hi }]}>{err}</Text></View>}
        {!!busy && <View style={s.row}><ActivityIndicator color={C.accent} /><Text style={s.dim}>  {busy}</Text></View>}

        {stage === 'home' && (
          <View>
            <Text style={s.h1}>Check your audio for location leaks before you post.</Text>
            <Text style={s.lead}>Faint aircraft passing overhead leave Doppler curves in a recording, and mains hum (50/60 Hz) carries its own clues. Matched against public records, these can narrow down where and when a clip was made. LeakScan finds them and helps you scrub them. It runs entirely in your browser: nothing is uploaded.</Text>
            <View style={[s.card, s.drop]}>
              <Text style={s.dropTitle}>Import audio or video</Text>
              <Text style={s.dim}>WAV, MP3, M4A, MP4… decoded locally on this device</Text>
              <View style={[s.row, { marginTop: 16, gap: 12 }]}>
                <Btn label="Choose file" onPress={() => run('Decoding & scanning…', async () => scan(await core.pickAudio()))} />
                <Btn kind="ghost" label="Try synthetic demo" onPress={() => run('Scanning demo…', () => scan(core.demoAudio()))} />
              </View>
              {!ready && core.supported && <Text style={[s.dim, { marginTop: 10 }]}>Loading on-device engine…</Text>}
            </View>
            <View style={[s.row, { gap: 12, flexWrap: 'wrap' }]}>
              {[['Detect', 'Doppler curves from aircraft and mains hum with harmonics, drawn on a spectrogram.'], ['Score', 'Number of passes, when they happen, and a relative "how narrow" rating.'], ['Scrub', 'Notch filters along each curve, hum removal, light noise, cut suggestions, re-scan, metadata-free export.']].map(([t, d]) => (
                <View key={t} style={[s.card, { flex: 1, minWidth: 220 }]}><Text style={s.h3}>{t}</Text><Text style={s.dim}>{d}</Text></View>))}
            </View>
          </View>
        )}

        {stage !== 'home' && orig && (
          <View>
            <View style={[s.row, { justifyContent: 'space-between', flexWrap: 'wrap' }]}>
              <Text style={s.h2}>{stage === 'results' ? 'Scan results' : 'Scrub: before / after'} <Text style={s.dim}>· {orig.a.name} · {fmt(orig.r.duration_s)}</Text></Text>
              <Btn kind="ghost" label="New file" onPress={() => { setStage('home'); setOrig(null); setClean(null); }} />
            </View>

            {stage === 'results' && (<>
              <View style={s.card}>
                <Text style={s.label}>Spectrogram 0–600 Hz · <Text style={{ color: C.accent }}>━ Doppler passes</Text>  <Text style={{ color: C.hum }}>┄ mains hum</Text></Text>
                <Image source={{ uri: orig.img }} style={s.spec} resizeMode="stretch" />
              </View>
              <View style={[s.row, { gap: 12, flexWrap: 'wrap', alignItems: 'stretch' }]}>
                <Score r={orig.r} title="Exposure score" />
                <View style={[s.card, { flex: 2, minWidth: 300 }]}>
                  <Text style={s.label}>What we found</Text>
                  {orig.r.flyovers.map((f, i) => (
                    <Text key={i} style={s.body}>✈ Pass {i + 1}: {fmt(f.start_s)}–{fmt(f.end_s)}, closest ≈ {fmt(f.peak_s)} ({f.f_start.toFixed(0)} → {f.f_end.toFixed(0)} Hz)</Text>))}
                  {orig.r.notes.map((n, i) => <Text key={'n' + i} style={s.dim}>• {n}</Text>)}
                  {orig.r.cuts.length > 0 && <Text style={[s.label, { marginTop: 10 }]}>Suggested cuts</Text>}
                  {orig.r.cuts.map((c, i) => <Text key={'c' + i} style={s.body}>✂ {fmt(c.start_s)}–{fmt(c.end_s)}</Text>)}
                  <View style={{ marginTop: 14 }}><Btn label="Scrub & re-scan" onPress={() => run('Scrubbing…', doScrub)} /></View>
                </View>
              </View>
            </>)}

            {stage === 'scrubbed' && clean && (<>
              <View style={[s.row, { gap: 12, flexWrap: 'wrap' }]}>
                {[['Before', orig], ['After', clean]].map(([t, x]: any) => (
                  <View key={t} style={[s.card, { flex: 1, minWidth: 300 }]}>
                    <Text style={s.label}>{t} · score <Text style={{ color: levelColor(x.r.level) }}>{x.r.score}/100 ({x.r.level})</Text> · {x.r.flyovers.length} passes · hum {x.r.hum.detected ? 'yes' : 'no'}</Text>
                    <Image source={{ uri: x.img }} style={s.specSmall} resizeMode="stretch" />
                    <AudioEl a={x.a} />
                  </View>))}
              </View>
              <View style={s.card}>
                <Text style={s.body}>Applied: notch filters along {orig.r.flyovers.reduce((n, f) => n + f.ridges.length, 0)} curve(s){orig.r.hum.detected ? `, ${orig.r.hum.base_hz} Hz hum + harmonics removed` : ''}, light noise bed (−52 dBFS), automatic re-scan. Export is a plain WAV with no metadata.</Text>
                <Text style={s.dim}>Scrubbing reduces these cues; it does not guarantee a clip can't be located. For sensitive material also apply the suggested cuts and avoid posting exact times.</Text>
                <View style={[s.row, { gap: 12, marginTop: 12 }]}>
                  <Btn label="Download clean WAV" onPress={() => core.exportWav(clean.a)} />
                  <Btn kind="ghost" label="Back to results" onPress={() => setStage('results')} />
                </View>
              </View>
            </>)}
          </View>
        )}
        <Text style={[s.dim, { marginTop: 24, textAlign: 'center' }]}>LeakScan v0.1 · Apache-2.0 core, AGPL-3.0 app · Reduces, does not guarantee.</Text>
      </ScrollView>
    </View>
  );
}

const s = StyleSheet.create({
  root: { flex: 1, backgroundColor: C.bg },
  page: { padding: 28, maxWidth: 1180, width: '100%', alignSelf: 'center' },
  header: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center', marginBottom: 22 },
  logo: { color: C.text, fontSize: 26, fontWeight: '800', letterSpacing: -0.5 },
  pill: { color: C.dim, borderColor: C.line, borderWidth: 1, borderRadius: 999, paddingHorizontal: 12, paddingVertical: 5, fontSize: 12 },
  h1: { color: C.text, fontSize: 38, fontWeight: '800', letterSpacing: -1, maxWidth: 760, marginBottom: 12 },
  h2: { color: C.text, fontSize: 22, fontWeight: '700', marginBottom: 12 },
  h3: { color: C.text, fontSize: 16, fontWeight: '700', marginBottom: 6 },
  lead: { color: C.dim, fontSize: 16, lineHeight: 24, maxWidth: 820, marginBottom: 20 },
  card: { backgroundColor: C.card, borderColor: C.line, borderWidth: 1, borderRadius: 14, padding: 18, marginBottom: 12 },
  drop: { borderStyle: 'dashed', borderColor: '#3a4180', alignItems: 'center', paddingVertical: 36 },
  dropTitle: { color: C.text, fontSize: 20, fontWeight: '700', marginBottom: 4 },
  row: { flexDirection: 'row', alignItems: 'center' },
  btn: { backgroundColor: C.accent, paddingHorizontal: 18, paddingVertical: 11, borderRadius: 10 },
  ghost: { backgroundColor: 'transparent', borderWidth: 1, borderColor: C.line },
  btnText: { color: '#1a0d06', fontWeight: '700' },
  label: { color: C.dim, fontSize: 12, textTransform: 'uppercase', letterSpacing: 0.8, marginBottom: 8 },
  body: { color: C.text, fontSize: 14, lineHeight: 22 },
  b: { fontWeight: '700' },
  dim: { color: C.dim, fontSize: 13, lineHeight: 20 },
  score: { fontSize: 52, fontWeight: '800' },
  scoreMax: { fontSize: 18, color: C.dim },
  spec: { width: '100%', aspectRatio: 1200 / 360, borderRadius: 8 },
  specSmall: { width: '100%', aspectRatio: 1200 / 360, borderRadius: 8 },
});
