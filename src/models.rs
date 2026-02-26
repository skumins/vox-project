use serde::Serialize;

#[derive(Serialize)]
pub struct TranscribeResponse {
    pub id: String,
    pub status: String,
}