// src/ui.ts
const logEl = document.getElementById('log') as HTMLPreElement;
const log = (m: string) => { logEl.textContent += m + '\n'; console.log(m); };

log('UI boot…');

// Import wasm-pack output as a real ESM (works since this file has a file URL)
import init, { analyze_from_pcm, ping } from "./pkg/audiofigma.js";

(async () => {
  try {
    log('Loading WASM…');
    // Pass the wasm URL explicitly to avoid relying on the glue’s import.meta heuristics
    await init(new URL('./pkg/audiofigma_bg.wasm', import.meta.url));
    log('WASM init() resolved');
    log('ping() = ' + ping());
  } catch (e: any) {
    log('WASM load/init failed: ' + (e?.message ?? e));
    console.error(e);
  }
})();

(document.getElementById('btn') as HTMLButtonElement).onclick = async () => {
  log('=== Analyze clicked ===');

  try {
    const fileEl = document.getElementById('file') as HTMLInputElement;
    const file = fileEl?.files?.[0];
    if (!file) { log('No file selected'); return; }

    const buf = await file.arrayBuffer();
    const AC = (window as any).AudioContext || (window as any).webkitAudioContext;
    const ac = new AC();
    const audioBuf = await ac.decodeAudioData(buf);
    const sr = audioBuf.sampleRate;

    let mono = audioBuf.getChannelData(0);
    if (audioBuf.numberOfChannels > 1) {
      const ch1 = audioBuf.getChannelData(1);
      const m = new Float32Array(mono.length);
      for (let i = 0; i < m.length; i++) m[i] = 0.5 * (mono[i] + ch1[i]);
      mono = m;
    }

    const packets = Number((document.getElementById('packets') as HTMLInputElement).value || 100);
    const result = analyze_from_pcm(mono, sr, packets);
    log(`WASM returned ${result.normalized.length} values (min=${result.minAvg}, max=${result.maxAvg})`);

    parent.postMessage({ pluginMessage: { type: 'analysis-ready', data: result } }, '*');
    log('Posted analysis-ready to main');
  } catch (e: any) {
    log('Analyze failed: ' + (e?.message ?? e));
    console.error(e);
  }
};
