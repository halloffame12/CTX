import { createClient } from "@supabase/supabase-js";
import type { Database } from "./types";
import { logger } from "./logger";

export type UserId = string;
export interface User {
  id: UserId;
  email: string;
  plan: "free" | "pro";
}
export enum Role {
  Admin = "admin",
  Member = "member",
}
export class UserService {
  private db: Database;
  constructor(db: Database) {
    this.db = db;
  }
  async getUser(id: UserId): Promise<User | null> {
    const { data } = await this.db.from("users").select("*").eq("id", id).single();
    return data as User | null;
  }
}
export function formatUser(u: User): string {
  return `${u.id}:${u.email}`;
}
