use std::fs::File;
use std::path::Path;

use symphonia::core::{
    audio::{Channels, SampleBuffer, SignalSpec},
    codecs::{DecoderOptions, CODEC_TYPE_NULL},
    errors::Error,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

#[derive(Debug, Clone)]
pub struct AudioAnalysis {
    pub volume_averages: Vec<f64>,
    pub bass_averages: Vec<f64>,
}

pub fn decode_and_analyze_audio(file_path: &str, num_packets: usize) -> Result<AudioAnalysis, Box<dyn std::error::Error>> {
    println!("cwd:  {}", std::env::current_dir()?.display());
    println!("open: {}", file_path);
    println!("exists? {}", Path::new(file_path).exists());

    let file = File::open(file_path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let probed = symphonia::default::get_probe().format(
        &Hint::new(),
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    let mut format = probed.format;

    // Pick first decodable audio track.
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("No decodable audio tracks found")?;
    let track_id = track.id;

    // Build decoder.
    let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    // Initial spec.
    let mut sample_rate = track.codec_params.sample_rate.unwrap_or(44_100);
    let mut chans: Channels = track.codec_params.channels.unwrap_or(Channels::FRONT_LEFT);

    let mut all_samples: Vec<f32> = Vec::new();
    let mut pkts = 0usize;

    println!("Decoding…");
    loop {
        match format.next_packet() {
            Ok(packet) => {
                if packet.track_id() != track_id {
                    continue;
                }
                pkts += 1;
                match decoder.decode(&packet) {
                    Ok(audio_buf) => {
                        let signal_spec = *audio_buf.spec();   // <-- local spec from this buffer
                        sample_rate = signal_spec.rate;
                        chans = signal_spec.channels;
                
                        let mut buf = SampleBuffer::<f32>::new(audio_buf.frames() as u64, signal_spec);
                        buf.copy_interleaved_ref(audio_buf);
                        all_samples.extend_from_slice(buf.samples());

                        if pkts % 50 == 0 {
                            println!("…{} packets, {} samples", pkts, all_samples.len());
                        }
                    }
                    Err(Error::DecodeError(e)) => {
                        eprintln!("recoverable DecodeError at packet {}: {}", pkts, e);
                        continue;
                    }
                    Err(Error::ResetRequired) => {
                        eprintln!("ResetRequired at packet {}, rebuilding decoder…", pkts);
                        let cp = &format
                            .tracks()
                            .iter()
                            .find(|t| t.id == track_id)
                            .ok_or("track gone")?
                            .codec_params;
                        decoder = symphonia::default::get_codecs().make(cp, &DecoderOptions::default())?;
                        continue;
                    }
                    Err(Error::IoError(_)) => break, // sometimes used to signal EOF
                    Err(e) => return Err(Box::new(e)),
                }
            }
            Err(Error::IoError(_)) => break, // EOF
            Err(e) => return Err(Box::new(e)),
        }
    }

    if all_samples.is_empty() {
        return Err("No audio samples decoded.".into());
    }

    // OPTIONAL: downmix to mono for analysis (more interpretable).
    let ch_count = chans.count();
    let mono: Vec<f32> = if ch_count > 1 {
        all_samples
            .chunks(ch_count as usize)
            .map(|frame| frame.iter().copied().map(|v| v as f64).sum::<f64>() / ch_count as f64)
            .map(|v| v as f32)
            .collect()
    } else {
        all_samples
    };

    // Chunking.
    let total = mono.len();
    let n = num_packets.max(1);
    let spp = (total / n).max(1);

    let mut volume_averages = Vec::with_capacity(n);
    let mut bass_averages = Vec::with_capacity(n);

    for i in 0..n {
        let start = i * spp;
        let end = if i == n - 1 { total } else { ((i + 1) * spp).min(total) };
        if start >= end { break; }
        let slice = &mono[start..end];
        volume_averages.push(calculate_rms(slice));
        bass_averages.push(calculate_bass_content(slice, sample_rate));
    }

    println!("Decoded {} samples @ {} Hz ({} packets).", total, sample_rate, pkts);
    Ok(AudioAnalysis { volume_averages, bass_averages })
}

fn calculate_rms(samples: &[f32]) -> f64 {
    if samples.is_empty() { return 0.0; }
    let sum_sq: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum_sq / samples.len() as f64).sqrt()
}

// O(n) first-order low-pass as a bass proxy (<~200 Hz).
fn calculate_bass_content(samples: &[f32], sample_rate: u32) -> f64 {
    if samples.is_empty() { return 0.0; }
    let fc = 200.0_f64;
    let fs = sample_rate as f64;
    let alpha = 1.0 - (-2.0 * std::f64::consts::PI * fc / fs).exp();

    let mut y = 0.0_f64;
    let mut acc_sq = 0.0_f64;
    for &x in samples {
        let x = x as f64;
        y += alpha * (x - y); // one-pole LPF
        acc_sq += y * y;
    }
    (acc_sq / samples.len() as f64).sqrt()
}

/// Combine per-packet volume & bass by averaging them,
/// then normalize so min -> `low` and max -> `high`.
fn normalized_packet_energy(
    volume: &[f64],
    bass: &[f64],
    low: f64,
    high: f64,
) -> (Vec<f64>, f64, f64) {
    assert_eq!(volume.len(), bass.len(), "vectors must be same length");
    let combined: Vec<f64> = volume.iter().zip(bass).map(|(v, b)| 0.5 * (v + b)).collect();

    let (mut mn, mut mx) = (f64::INFINITY, f64::NEG_INFINITY);
    for &x in &combined {
        if x < mn { mn = x; }
        if x > mx { mx = x; }
    }

    // Map linearly: y = low + (x - mn) * (high - low) / (mx - mn)
    let norm = if (mx - mn).abs() < f64::EPSILON {
        // Edge case: all the same → choose the top bound
        vec![high; combined.len()]
    } else {
        let span = high - low;
        combined.into_iter().map(|x| low + (x - mn) * span / (mx - mn)).collect()
    };

    (norm, mn, mx)
}

fn main() {
    let path = "audiofolder/devilinanewdress.mp3";

    // Bind the analysis so it's available below.
    let analysis = match decode_and_analyze_audio(path, 100) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Analyze failed: {e}");
            return;
        }
    };

    // Now compute normalized combined scores.
    let (normalized, min_avg, max_avg) = normalized_packet_energy(
        &analysis.volume_averages,
        &analysis.bass_averages,
        0.2,
        1.0,
    );

    println!("normalized per-packet energy: {:?}", normalized);
}