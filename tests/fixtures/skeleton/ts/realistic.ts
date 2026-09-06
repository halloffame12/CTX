import { createClient } from "@supabase/supabase-js";
import { z } from "zod";

const supabase = createClient(process.env.URL!, process.env.KEY!);

export async function fetchUserProfile(userId: string) {
  const { data, error } = await supabase
    .from("profiles")
    .select("id, email, display_name, avatar_url, created_at")
    .eq("id", userId)
    .single();

  if (error) {
    console.error("failed to fetch profile", error);
    throw new Error(`profile lookup failed: ${error.message}`);
  }

  const parsed = profileSchema.safeParse(data);
  if (!parsed.success) {
    console.warn("invalid profile shape", parsed.error.issues);
    return null;
  }

  return {
    id: parsed.data.id,
    email: parsed.data.email,
    displayName: parsed.data.display_name,
    avatarUrl: parsed.data.avatar_url,
    createdAt: parsed.data.created_at,
  };
}

export async function updateUserProfile(userId: string, patch: Partial<ProfilePatch>) {
  const updates = {};
  if (patch.displayName !== undefined) updates["display_name"] = patch.displayName;
  if (patch.avatarUrl !== undefined) updates["avatar_url"] = patch.avatarUrl;

  const { data, error } = await supabase
    .from("profiles")
    .update(updates)
    .eq("id", userId)
    .select("id, email")
    .single();

  if (error) {
    throw new Error(`profile update failed: ${error.message}`);
  }
  return data;
}

export async function listUserProfiles(opts: { limit?: number; offset?: number } = {}) {
  const limit = Math.min(opts.limit ?? 20, 100);
  const offset = opts.offset ?? 0;
  const { data, error, count } = await supabase
    .from("profiles")
    .select("id, email, display_name, avatar_url", { count: "exact" })
    .range(offset, offset + limit - 1)
    .order("created_at", { ascending: false });

  if (error) {
    throw new Error(`profile list failed: ${error.message}`);
  }
  return { items: data ?? [], total: count ?? 0 };
}