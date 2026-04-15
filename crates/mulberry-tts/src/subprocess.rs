use std::process::Command;

use tracing::{info, warn};

use crate::engine::{TtsEngine, TtsRequest, TtsResult};
use crate::error::TtsError;

/// TTS engine that delegates synthesis to an external process (e.g. espeak-ng).
pub struct SubprocessTtsEngine {
    command: String,
    args: Vec<String>,
    #[allow(dead_code)]
    sample_rate: u32,
}

impl SubprocessTtsEngine {
    pub fn new(command: &str, sample_rate: u32) -> Self {
        Self {
            command: command.to_string(),
            args: Vec::new(),
            sample_rate,
        }
    }

    /// Convenience constructor for espeak-ng.
    pub fn espeak(sample_rate: u32) -> Self {
        Self {
            command: "espeak-ng".to_string(),
            args: Vec::new(),
            sample_rate,
        }
    }

    /// Parse a WAV byte buffer into f32 samples.
    /// Supports 16-bit PCM WAV files (the format espeak-ng produces).
    fn parse_wav(data: &[u8]) -> Result<(Vec<f32>, u32), TtsError> {
        // Minimal WAV header parsing
        if data.len() < 44 {
            return Err(TtsError::SynthesisFailed(
                "WAV data too short".to_string(),
            ));
        }

        // Verify RIFF header
        if &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
            return Err(TtsError::SynthesisFailed(
                "invalid WAV header".to_string(),
            ));
        }

        // Find the "fmt " and "data" chunks by scanning
        let mut pos = 12;
        let mut sample_rate = 0u32;
        let mut bits_per_sample = 0u16;
        let mut num_channels = 0u16;
        let mut data_start = 0;
        let mut data_size = 0usize;

        while pos + 8 <= data.len() {
            let chunk_id = &data[pos..pos + 4];
            let chunk_size =
                u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
                    as usize;

            if chunk_id == b"fmt " && pos + 8 + chunk_size <= data.len() {
                num_channels =
                    u16::from_le_bytes([data[pos + 10], data[pos + 11]]);
                sample_rate = u32::from_le_bytes([
                    data[pos + 12],
                    data[pos + 13],
                    data[pos + 14],
                    data[pos + 15],
                ]);
                bits_per_sample =
                    u16::from_le_bytes([data[pos + 22], data[pos + 23]]);
            } else if chunk_id == b"data" {
                data_start = pos + 8;
                data_size = chunk_size;
                break;
            }

            pos += 8 + chunk_size;
            // WAV chunks are word-aligned
            if chunk_size % 2 != 0 {
                pos += 1;
            }
        }

        if data_start == 0 || sample_rate == 0 {
            return Err(TtsError::SynthesisFailed(
                "could not find WAV data chunk".to_string(),
            ));
        }

        let actual_data_len = data_size.min(data.len() - data_start);
        let raw = &data[data_start..data_start + actual_data_len];

        let samples: Vec<f32> = match bits_per_sample {
            16 => raw
                .chunks_exact(2)
                .map(|c| {
                    let s = i16::from_le_bytes([c[0], c[1]]);
                    s as f32 / i16::MAX as f32
                })
                .collect(),
            8 => raw.iter().map(|&b| (b as f32 - 128.0) / 128.0).collect(),
            _ => {
                return Err(TtsError::SynthesisFailed(format!(
                    "unsupported bits per sample: {}",
                    bits_per_sample
                )));
            }
        };

        // Mix to mono if stereo
        let mono = if num_channels == 2 {
            samples
                .chunks_exact(2)
                .map(|pair| (pair[0] + pair[1]) * 0.5)
                .collect()
        } else {
            samples
        };

        Ok((mono, sample_rate))
    }
}

impl TtsEngine for SubprocessTtsEngine {
    fn synthesize(&self, request: &TtsRequest) -> Result<TtsResult, TtsError> {
        if request.text.is_empty() {
            return Err(TtsError::InvalidInput("empty text".to_string()));
        }

        // espeak-ng: speed is in words per minute (default ~175)
        let wpm = (175.0 * request.speed) as u32;
        let pitch = (50.0 * request.pitch) as u32; // espeak pitch: 0-99, default 50

        let mut cmd = Command::new(&self.command);
        cmd.arg("--stdout")
            .arg("-s")
            .arg(wpm.to_string())
            .arg("-p")
            .arg(pitch.to_string());

        for arg in &self.args {
            cmd.arg(arg);
        }

        cmd.arg(&request.text);

        info!(
            command = %self.command,
            text = %request.text,
            "Running TTS subprocess"
        );

        let output = cmd.output().map_err(|e| {
            TtsError::ProcessError(format!(
                "failed to execute '{}': {}",
                self.command, e
            ))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TtsError::ProcessError(format!(
                "'{}' exited with {}: {}",
                self.command,
                output.status,
                stderr.trim()
            )));
        }

        let (samples, wav_sample_rate) = Self::parse_wav(&output.stdout)?;
        let duration_secs = if wav_sample_rate > 0 {
            samples.len() as f32 / wav_sample_rate as f32
        } else {
            0.0
        };

        Ok(TtsResult {
            samples,
            sample_rate: wav_sample_rate,
            duration_secs,
        })
    }

    fn name(&self) -> &str {
        &self.command
    }

    fn is_available(&self) -> bool {
        match Command::new(&self.command).arg("--version").output() {
            Ok(output) => {
                let available = output.status.success();
                if !available {
                    warn!(command = %self.command, "TTS command not available");
                }
                available
            }
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subprocess_engine_new() {
        let engine = SubprocessTtsEngine::new("espeak-ng", 44100);
        assert_eq!(engine.name(), "espeak-ng");
        assert_eq!(engine.sample_rate, 44100);
    }

    #[test]
    fn test_subprocess_espeak_constructor() {
        let engine = SubprocessTtsEngine::espeak(22050);
        assert_eq!(engine.name(), "espeak-ng");
        assert_eq!(engine.sample_rate, 22050);
    }

    #[test]
    fn test_parse_wav_too_short() {
        let result = SubprocessTtsEngine::parse_wav(&[0u8; 10]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_wav_invalid_header() {
        let data = vec![0u8; 44];
        let result = SubprocessTtsEngine::parse_wav(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_wav_valid_16bit() {
        // Construct a minimal valid 16-bit PCM WAV
        let sample_rate: u32 = 22050;
        let num_channels: u16 = 1;
        let bits_per_sample: u16 = 16;
        let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
        let block_align = num_channels * bits_per_sample / 8;

        // Two samples: 0 and i16::MAX
        let audio_data: Vec<u8> = vec![0, 0, 0xFF, 0x7F];
        let data_size = audio_data.len() as u32;
        let file_size = 36 + data_size;

        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&file_size.to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        // fmt chunk
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&num_channels.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());
        // data chunk
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());
        wav.extend_from_slice(&audio_data);

        let (samples, sr) = SubprocessTtsEngine::parse_wav(&wav).unwrap();
        assert_eq!(sr, 22050);
        assert_eq!(samples.len(), 2);
        assert!((samples[0] - 0.0).abs() < 0.001);
        assert!((samples[1] - 1.0).abs() < 0.001);
    }
}
