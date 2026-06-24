use crate::audio::state::AudioBuffer;
use minimp3::{Decoder, Error as Mp3Error};
use std::fs::File;
use std::path::Path;

pub fn load_wav(path: &Path) -> Result<AudioBuffer, String> {
    let mut reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
    let spec = reader.spec();

    let samples: Result<Vec<f32>, _> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect(),
        hound::SampleFormat::Int => {
            if spec.bits_per_sample == 16 {
                reader
                    .samples::<i16>()
                    .map(|s| s.map(|v| v as f32 / i16::MAX as f32))
                    .collect()
            } else if spec.bits_per_sample == 24 {
                reader
                    .samples::<i32>()
                    .map(|s| s.map(|v| v as f32 / 8388608.0))
                    .collect()
            } else if spec.bits_per_sample == 32 {
                reader
                    .samples::<i32>()
                    .map(|s| s.map(|v| v as f32 / i32::MAX as f32))
                    .collect()
            } else {
                return Err(format!("Unsupported bit depth: {}", spec.bits_per_sample));
            }
        }
    };

    let samples = samples.map_err(|e| e.to_string())?;

    Ok(AudioBuffer {
        samples,
        channels: spec.channels,
        sample_rate: spec.sample_rate,
    })
}

pub fn load_mp3(path: &Path) -> Result<AudioBuffer, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut decoder = Decoder::new(file);

    let mut samples = Vec::new();
    let mut channels = 0;
    let mut sample_rate = 0;

    loop {
        match decoder.next_frame() {
            Ok(frame) => {
                if channels == 0 {
                    channels = frame.channels as u16;
                    sample_rate = frame.sample_rate as u32;
                }

                for sample in frame.data {
                    samples.push(sample as f32 / i16::MAX as f32);
                }
            }
            Err(Mp3Error::Eof) => break,
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok(AudioBuffer {
        samples,
        channels,
        sample_rate,
    })
}

pub fn load_file(path: &Path) -> Result<AudioBuffer, String> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    match extension.as_str() {
        "wav" => load_wav(path),
        "mp3" => load_mp3(path),
        _ => Err(format!("Unsupported file format: {}", extension)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hound::{SampleFormat, WavSpec, WavWriter};
    use std::env::temp_dir;
    use std::fs;

    #[test]
    fn test_load_wav_valid() {
        let mut path = temp_dir();
        path.push("test_audio.wav");

        let spec = WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::create(&path, spec).unwrap();
        writer.write_sample(i16::MAX).unwrap();
        writer.write_sample(0_i16).unwrap();
        writer.write_sample(-i16::MAX).unwrap();
        writer.finalize().unwrap();

        let result = load_wav(&path).unwrap();
        assert_eq!(result.channels, 1);
        assert_eq!(result.sample_rate, 44100);
        assert_eq!(result.samples.len(), 3);
        assert!((result.samples[0] - 1.0).abs() < 0.001);
        assert_eq!(result.samples[1], 0.0);
        assert!((result.samples[2] - -1.0).abs() < 0.001);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_load_file_unsupported_extension() {
        let mut path = temp_dir();
        path.push("test.txt");
        let result = load_file(&path);
        assert!(result.is_err());
    }
}
