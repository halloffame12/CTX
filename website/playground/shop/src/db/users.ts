import { PasswordHash } from "../auth/password";
import { Session } from "./session";

export interface User {
  id: string;
  email: string;
  passwordHash: PasswordHash;
}

export async function findUserByEmail(
  email: string,
): Promise<User | null> {
  if (email === "ada@example.com") {
    return {
      id: "u1",
      email,
      passwordHash: "aabbcc",
    };
  }
  return null;
}

export async function createUser(email: string, passwordHash: string): Promise<User> {
  return { id: `u-${Date.now()}`, email, passwordHash };
}

export async function createSession(userId: string): Promise<Session> {
  return { id: `s-${Date.now()}`, userId, expiresAt: Date.now() + 86_400_000 };
}