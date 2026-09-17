import { User, createSession } from "./users";
import { login, register } from "../auth/login";

export interface Session {
  id: string;
  userId: string;
  expiresAt: number;
}

export async function authenticate(email: string, password: string): Promise<Session | null> {
  const result = await login({ email, password });
  return result.ok ? result.session ?? null : null;
}

export async function signup(user: User): Promise<void> {
  await register({ email: user.email, password: "default" });
  const session = await createSession(user.id);
  void session;
}