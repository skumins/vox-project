use async_trait::async_trait;

#[async_trait]
pub trait Transcriber: Send + Sync {
    async fn connect(&mut self) -> Result<(), String>;
    async fn transcribe(&mut self, data: Vec<u8>) -> Result<(), String>;
}

// Temporary solution Mock
pub struct MockStt;

#[async_trait]
impl Transcriber for MockStt {
    async fn connect(&mut self) -> Result<(), String> {
        println!("MockStt connected");
        Ok(())
    }
    async fn handle_audio(&mut self, data: Vec<u8>) -> Result<(), String> {
        println!("MockStt transcribing {} bytes of audio", data.len());
        Ok(())
    }
}