import { useState, useEffect } from "react";

type User = {
  id: string;
  email: string;
  name: string;
  avatar_url: string | null;
};

type AuthState =
  | { status: "loading" }
  | { status: "authenticated"; user: User }
  | { status: "unauthenticated" };

export function useAuth() {
  const [authState, setAuthState] = useState<AuthState>({ status: "loading" });

  useEffect(() => {
    async function checkAuth() {
      try {
        const response = await fetch("/api/me", { credentials: "include" });

        if (response.ok) {
          const user: User = await response.json();
          setAuthState({ status: "authenticated", user });
        } else {
          setAuthState({ status: "unauthenticated" });
        }
      } catch {
        setAuthState({ status: "unauthenticated" });
      }
    }
    checkAuth();
  }, []);

  async function logout() {
    await fetch("/auth/logout", {
      method: "POST",
      credentials: "include",
    });
    setAuthState({ status: "unauthenticated" });
  }
  return { authState, logout };
}
