use async_trait::async_trait;

pub struct AudioConfig {
    pub sample_rate: u32,
}
#[async_trait]
pub trait Transcriber: Send + Sync {
    async fn connect(&mut self, config: AudioConfig) -> Result<(), String>;
    async fn handle_audio(&mut self, data: Vec<f32>) -> Result<(), String>;
}

// Temporary solution Mock
pub struct MockStt;

#[async_trait]
impl Transcriber for MockStt {
    async fn connect(&mut self, _config: AudioConfig) -> Result<(), String> {
        println!("MockStt connected");
        Ok(())
    }
    async fn handle_audio(&mut self, data: Vec<f32>) -> Result<(), String> {
        println!("MockStt transcribing {} bytes of audio", data.len());
        Ok(())
    }
}