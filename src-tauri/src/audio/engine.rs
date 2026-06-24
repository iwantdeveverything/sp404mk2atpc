use crate::audio::state::{AudioState, AudioStateInner};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};

pub fn start_audio_engine(state: AudioState) -> Result<Stream, String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No output device available")?;
    let config = device.default_output_config().map_err(|e| e.to_string())?;

    let stream_config: StreamConfig = config.clone().into();
    let channels = stream_config.channels as usize;
    let sample_rate = stream_config.sample_rate;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_output_stream(
            stream_config.clone(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, sample_rate, &state)
            },
            err_fn,
            None,
        ),
        _ => return Err("Unsupported sample format".to_string()),
    }
    .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;

    Ok(stream)
}

fn write_data(output: &mut [f32], channels: usize, target_sample_rate: u32, state: &AudioState) {
    // Clear the output buffer first
    for sample in output.iter_mut() {
        *sample = 0.0;
    }

    let mut inner = match state.inner.lock() {
        Ok(guard) => guard,
        Err(_) => return, // Handle poisoned mutex gracefully in the audio thread
    };

    let mut finished_events = Vec::new();

    let AudioStateInner {
        buffers,
        active_events,
    } = &mut *inner;

    // The output buffer is interleaved: [L, R, L, R, ...]
    for frame in output.chunks_mut(channels) {
        let mut frame_mix = vec![0.0; channels];

        for (i, event) in active_events.iter_mut().enumerate() {
            if let Some(buffer) = buffers.get(&event.pad_id) {
                let ratio = buffer.sample_rate as f32 / target_sample_rate as f32;
                let index_f = event.position;
                let index = index_f as usize;

                if index * (buffer.channels as usize) >= buffer.samples.len() {
                    finished_events.push(i);
                    continue;
                }

                // Basic resampling with nearest neighbor interpolation and mixing
                for c in 0..channels {
                    let source_c = if c < buffer.channels as usize { c } else { 0 }; // Mono to stereo fallback
                    let sample_idx = index * (buffer.channels as usize) + source_c;

                    if sample_idx < buffer.samples.len() {
                        frame_mix[c] += buffer.samples[sample_idx] * event.volume;
                    }
                }

                event.position += ratio;
            } else {
                finished_events.push(i);
            }
        }

        // Apply mix to frame with clipping protection
        for (c, sample) in frame.iter_mut().enumerate() {
            *sample = frame_mix[c].clamp(-1.0_f32, 1.0_f32);
        }
    }

    // Remove finished events in reverse order to preserve indices
    finished_events.sort_unstable_by(|a, b| b.cmp(a));
    finished_events.dedup();
    for i in finished_events {
        if i < active_events.len() {
            active_events.remove(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::state::{AudioBuffer, AudioState};

    #[test]
    fn test_write_data_mixing() {
        let state = AudioState::new();
        state.add_buffer(
            0,
            AudioBuffer {
                samples: vec![1.0, 1.0, 1.0, 1.0],
                channels: 1,
                sample_rate: 44100,
            },
        );
        state.add_buffer(
            1,
            AudioBuffer {
                samples: vec![-0.5, -0.5, -0.5, -0.5],
                channels: 1,
                sample_rate: 44100,
            },
        );

        state.trigger_pad(0);
        state.trigger_pad(1);

        let mut output = vec![0.0; 4]; // 2 frames of stereo
        write_data(&mut output, 2, 44100, &state);

        assert_eq!(output, vec![0.5, 0.5, 0.5, 0.5]);
    }

    #[test]
    fn test_write_data_resampling() {
        let state = AudioState::new();
        state.add_buffer(
            0,
            AudioBuffer {
                samples: vec![0.1, 0.2, 0.3, 0.4],
                channels: 1,
                sample_rate: 22050, // Half the target rate
            },
        );

        state.trigger_pad(0);

        let mut output = vec![0.0; 8]; // 4 frames of stereo
        write_data(&mut output, 2, 44100, &state);

        // Ratio is 0.5.
        // Frame 1: index 0 (int 0) -> buffer[0] -> 0.1
        // Frame 2: index 0.5 (int 0) -> buffer[0] -> 0.1
        // Frame 3: index 1.0 (int 1) -> buffer[1] -> 0.2
        // Frame 4: index 1.5 (int 1) -> buffer[1] -> 0.2
        assert_eq!(output, vec![0.1, 0.1, 0.1, 0.1, 0.2, 0.2, 0.2, 0.2]);
    }
}
