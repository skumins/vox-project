# VOXA — Real-Time Lecture Transcription & Notes
 
> Record lectures. Get transcripts. Create notes automatically.

<img width="1254" height="705" alt="voxa" src="https://github.com/user-attachments/assets/2c05fcbd-a9e3-48de-b003-1dfbe7836e9d" />


[![Rust](https://img.shields.io/badge/Backend-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/Frontend-React-61DAFB?logo=react)](https://react.dev/)
[![PostgreSQL](https://img.shields.io/badge/Database-PostgreSQL-4169E1?logo=postgresql)](https://www.postgresql.org/)
 
---
 
VOXA transcribes audio in real time and generates structured notes in Markdown using an LLM.
Each user connects their own API keys, which makes the service free, independent of any specific provider, and gives full control over the data.
 
## Features
 
- **Real-time transcription** via microphone with WebSocket streaming
- **Voice Activity Detection** runs entirely in the browser (WASM). Audio is sent to the API only when speech is detected, which keeps API usage low
- **AI-generated notes** structured in Markdown, produced by an LLM after the session
- **Multilingual** – transcribe in one language, get notes in another
- **Google OAuth 2.0** authentication with secure encrypted sessions
- **User-provided API keys** – each user connects their own Deepgram and OpenRouter credentials
## Tech Stack
 
| Layer | Technology |
|---|---|
| Backend | Rust, Axum, Tokio, SQLx |
| Frontend | TypeScript, React, Vite |
| Database | PostgreSQL |
| Speech-to-Text | Deepgram nova-2 |
| LLM | OpenRouter |
| VAD | Silero v5 (`@ricky0123/vad-react`, runs in browser) |
| Auth | Google OAuth 2.0 with PKCE and encrypted session cookies |
 
## Run Locally
 
**Requirements:** Rust (stable), Node.js 18+, PostgreSQL 16+
 
**1. Clone the repository**
 
```bash
git clone https://github.com/skumins/vox-project.git
cd vox-project
```
 
**2. Create a `.env` file in the project root**
 
```dotenv
DATABASE_URL=postgresql://voxa_user:your_password@localhost:5432/voxa_db
 
DEEPGRAM_API_KEY=your_key
OPENROUTER_API_KEY=your_key
OPENROUTER_MODEL=openrouter/free
 
GOOGLE_CLIENT_ID=your_client_id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=your_secret
GOOGLE_REDIRECT_URL=http://localhost:3000/auth/google/callback
 
# 64 hex characters - used to encrypt stored API keys
ENCRYPTION_KEY=your_encryption_key
 
# 128 hex characters - used to sign session cookies
COOKIE_SECRET=your_cookie_secret
```
 
Generate the keys using `openssl` (works on Windows with Git Bash, macOS, and Linux):
 
```bash
openssl rand -hex 32   # -> ENCRYPTION_KEY
openssl rand -hex 64   # -> COOKIE_SECRET
```
 
**3. Set up the database**
 
```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx migrate run
```
 
**4. Start the backend**
 
```bash
cargo run
```
 
**5. Start the frontend**
 
```bash
cd frontend
npm install
npm run dev
```
 
Open `http://localhost:5173` in the browser.
 
> **Google OAuth:** create a project at [console.cloud.google.com](https://console.cloud.google.com), go to APIs & Services, create OAuth credentials, and add `http://localhost:3000/auth/google/callback` as an authorized redirect URI.
 
## Roadmap
 
- [ ] User profile with API key management
- [ ] Multiple STT and LLM provider support with user choice
- [ ] Notes archive with editing and export to PDF and Markdown
- [ ] Real-time speech translation with audio playback
- [ ] Redis for session storage and translation pipeline
- [ ] Desktop version via Tauri with local models (Whisper, NLLB, Piper TTS)
