use wasm_bindgen::prelude::*;
use js_sys::Float32Array;
use serde::Serialize;
use web_sys::console;

#[wasm_bindgen(start)]
pub fn wasm_start() {
    // so Rust panics show in the UI console
    console_error_panic_hook::set_once();
    console::log_1(&"audiofigma: WASM initialized".into());
}

#[wasm_bindgen]
pub fn ping() -> i32 {
    console::log_1(&"audiofigma: ping() called inside WASM".into());
    42
}
#[wasm_bindgen]
pub fn analyze_from_pcm(samples: js_sys::Float32Array, sample_rate: u32, packets: usize) -> JsValue {
    console::log_1(&format!("audiofigma: analyze_from_pcm called with {} samples @ {}Hz, {} packets", 
                             samples.length(), sample_rate, packets).into());
    
    let mut v = vec![0f32; samples.length() as usize];
    samples.copy_to(&mut v[..]);
    console::log_1(&format!("audiofigma: Copied {} samples to Rust Vec", v.len()).into());

    // … your existing packetize + RMS + bass + normalize …
    console::log_1(&"audiofigma: Starting analysis...".into());
    let (volume, bass) = analyze_vectors(&v, sample_rate, packets.max(1));
    console::log_1(&format!("audiofigma: Analysis complete: {} volume + {} bass values", volume.len(), bass.len()).into());
    
    let (normalized, min_avg, max_avg) = normalized_packet_energy(&volume, &bass, 0.2, 1.0);
    console::log_1(&format!("audiofigma: Normalized to {} values (min: {:.4}, max: {:.4})", 
                             normalized.len(), min_avg, max_avg).into());

    let result = serde_wasm_bindgen::to_value(&WasmResp {
        normalized, minAvg: min_avg, maxAvg: max_avg
    }).unwrap();
    
    console::log_1(&"audiofigma: Returning WASM result".into());
    result
}

#[derive(Serialize)]
struct WasmResp { normalized: Vec<f64>, minAvg: f64, maxAvg: f64 }

// ===== analysis reused from your backend (PCM already mono or stereo-avg in UI) =====

fn analyze_vectors(mono: &[f32], sample_rate: u32, packets: usize) -> (Vec<f64>, Vec<f64>) {
    let total = mono.len();
    let spp = (total / packets).max(1);
    console::log_1(&format!("audiofigma: Packetizing {} samples into {} packets ({} samples/packet)", 
                             total, packets, spp).into());
    
    let mut volume = Vec::with_capacity(packets);
    let mut bass = Vec::with_capacity(packets);

    for i in 0..packets {
        let start = i * spp;
        let end = if i == packets - 1 { total } else { ((i + 1) * spp).min(total) };
        if start >= end { break; }
        let slice = &mono[start..end];
        volume.push(calculate_rms(slice));
        bass.push(calculate_bass_content(slice, sample_rate));
    }
    
    console::log_1(&format!("audiofigma: Generated {} volume packets and {} bass packets", 
                             volume.len(), bass.len()).into());
    (volume, bass)
}

fn calculate_rms(samples: &[f32]) -> f64 {
    if samples.is_empty() { return 0.0; }
    let sum_sq: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum_sq / samples.len() as f64).sqrt()
}

// O(n) first-order low-pass (≈ <200 Hz content)
fn calculate_bass_content(samples: &[f32], sample_rate: u32) -> f64 {
    if samples.is_empty() { return 0.0; }
    let fc = 200.0_f64;
    let fs = sample_rate as f64;
    let alpha = 1.0 - (-2.0 * std::f64::consts::PI * fc / fs).exp();

    let mut y = 0.0_f64;
    let mut acc_sq = 0.0_f64;
    for &x in samples {
        let x = x as f64;
        y += alpha * (x - y);
        acc_sq += y * y;
    }
    (acc_sq / samples.len() as f64).sqrt()
}

fn normalized_packet_energy(volume: &[f64], bass: &[f64], low: f64, high: f64) -> (Vec<f64>, f64, f64) {
    assert_eq!(volume.len(), bass.len());
    let combined: Vec<f64> = volume.iter().zip(bass).map(|(v, b)| 0.5 * (v + b)).collect();
    let (mut mn, mut mx) = (f64::INFINITY, f64::NEG_INFINITY);
    for &x in &combined { if x < mn { mn = x; } if x > mx { mx = x; } }
    if (mx - mn).abs() < f64::EPSILON {
        return (vec![high; combined.len()], mn, mx);
    }
    let span = high - low;
    let norm = combined.into_iter().map(|x| low + (x - mn) * span / (mx - mn)).collect();
    (norm, mn, mx)
}