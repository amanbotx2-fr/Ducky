import "server-only";

import { createClient, type SupabaseClient } from "@supabase/supabase-js";

export type DownloadEvent = {
  platform: "mac" | "windows" | "linux";
  releaseTag: string;
  assetName: string;
  occurredAt: string;
};

export interface DownloadTracker {
  record(event: DownloadEvent): Promise<void>;
}

let supabaseClient: SupabaseClient | undefined;

function getSupabaseClient(): SupabaseClient {
  if (supabaseClient) {
    return supabaseClient;
  }

  const supabaseUrl = process.env.SUPABASE_URL;
  const serviceRoleKey = process.env.SUPABASE_SERVICE_ROLE_KEY;

  if (!supabaseUrl || !serviceRoleKey) {
    throw new Error("Supabase download tracking is not configured");
  }

  supabaseClient = createClient(supabaseUrl, serviceRoleKey, {
    auth: {
      autoRefreshToken: false,
      detectSessionInUrl: false,
      persistSession: false,
    },
  });

  return supabaseClient;
}

class SupabaseDownloadTracker implements DownloadTracker {
  async record(event: DownloadEvent): Promise<void> {
    const { error } = await getSupabaseClient().from("downloads").insert({
      platform: event.platform,
      version: event.releaseTag,
    });

    if (error) {
      throw new Error(`Supabase download insert failed: ${error.message}`);
    }
  }
}

const tracker: DownloadTracker = new SupabaseDownloadTracker();

export async function recordDownload(event: DownloadEvent): Promise<void> {
  try {
    await tracker.record(event);
  } catch (error) {
    console.error("[ducky-download] Tracking failed", error);
  }
}
