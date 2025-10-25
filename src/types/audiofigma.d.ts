// src/types/audiofigma.d.ts
declare module "./pkg/audiofigma.js" {
    // The wasm-pack default export (init). We accept several input types.
    const init: (
      input?:
        | string
        | URL
        | Request
        | Response
        | ArrayBuffer
        | WebAssembly.Module
    ) => Promise<any>;
  
    // Your Rust exports
    export function analyze_from_pcm(
      samples: Float32Array,
      sample_rate: number,
      packets: number
    ): { normalized: number[]; minAvg: number; maxAvg: number };
  
    export function ping(): number;
  
    export default init;
  }
  