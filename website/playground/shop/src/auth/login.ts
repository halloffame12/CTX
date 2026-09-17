import { hashPassword, verifyPassword } from "./password";
import { findUserByEmail, createSession } from "../db/users";
import { Session } from "../db/session";

export interface LoginInput {
  email: string;
  password: string;
}

export interface LoginResult {
  ok: boolean;
  session?: Session;
  error?: string;
}

export async function login(input: LoginInput): Promise<LoginResult> {
  const user = await findUserByEmail(input.email);
  if (!user) {
    return { ok: false, error: "no user with that email" };
  }
  const valid = await verifyPassword(input.password, user.passwordHash);
  if (!valid) {
    return { ok: false, error: "incorrect password" };
  }
  const session = await createSession(user.id);
  return { ok: true, session };
}

export async function register(input: LoginInput): Promise<LoginResult> {
  const passwordHash = await hashPassword(input.password);
  return { ok: true, error: undefined, session: undefined, passwordHash };
}