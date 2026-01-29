use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crate::stt::{AudioConfig, Transcriber};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn start_capture(stt: Arc<Mutex<dyn Transcriber>>) -> Result<(), String> {
    // Setup the host
    let host = cpal::default_host();
    // Select default input device
    let device = host.default_output_device()
        .ok_or("No input device found")?;

    // Get supported config
    let config = device.default_input_config()
        .map_err(|e| e.to_string())?;

    let audio_config = AudioConfig {
        sample_rate: config.sample_rate().0,
    };

    // Initialize STT connection
    stt.lock().await.connect(audio_config).await?;
    let stt_clone = stt.clone();

    // Build the input stream
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &_| {
            // Audio data input
            let stt_clone = stt_clone.clone();
            let chunk = data.to_vec();

            // Processing without blocking the audio stream
            tokio::spawn(async move {
                let mut lock = stt_inner.lock().await;
                let _ = lock.handle_audio(chunk).await;
            });
        },
        |err| eprintln!("Audio stream error: {}", err),
        None
    ).map_err(|e| e.to_string())?;

    // Start recording
    stream.play().map_err(|e| e.to_string())?;

    // For testing 10s
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    Ok(())
}