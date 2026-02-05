use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crate::stt::{AudioConfig, Transcriber};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

pub async fn start_capture(stt: Arc<Mutex<dyn Transcriber>>) -> Result<(), String> {
    // Setup the host
    let host = cpal::default_host();
    // Select default input device
    let device = host.default_input_device()
        .ok_or("No input device found")?;

    // Get supported config
    let config = device.default_input_config()
        .map_err(|e| e.to_string())?;

    let audio_config = AudioConfig {
        sample_rate: config.sample_rate(),
    };

    // Initialize STT connection
    stt.lock().await.connect(audio_config).await?;
    let(tx, mut rx) = mpsc::channel::<Vec<f32>>(100);

    let stt_receiver = stt.clone();
    tokio::spawn(async move {
        while let Some(audio_data) = rx.recv().await {
            let mut lock = stt_receiver.lock().await;
            if let Err(e) = lock.handle_audio(audio_data).await {
                eprintln!("Error STT processing: {}", e);
            }
        }
    });

    // Build the input stream
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &_| {
            // Audio data input
            let _ = tx.blocking_send(data.to_vec());
        },
        |err| eprintln!("Error Audio: {}", err),
        None
    ).map_err(|e| e.to_string())?;

    // Start recording
    stream.play().map_err(|e| e.to_string())?;

    println!("Audio recording started; press Ctrl+C to stop");
    tokio::time::sleep(std::time::Duration::from_secs(100)).await;
    Ok(())
}