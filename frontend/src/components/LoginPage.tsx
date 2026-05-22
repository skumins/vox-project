export function LoginPage() {
  return (
    <div className="login-page">
      <div className="login-card">
        <h1>VOXA</h1>
        <p>Lecture transcription and notes</p>
        <a href="/auth/google" className="btn-google">
          Sing in with Google
        </a>
      </div>
    </div>
  );
}
