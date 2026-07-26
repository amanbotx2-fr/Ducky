import "server-only";

import { createClient, type SupabaseClient } from "@supabase/supabase-js";

let serverClient: SupabaseClient | undefined;

export function getServerSupabaseClient(): SupabaseClient {
  if (serverClient) {
    return serverClient;
  }

  const supabaseUrl = process.env.SUPABASE_URL;
  const serviceRoleKey = process.env.SUPABASE_SERVICE_ROLE_KEY;
  const missingVariables = [
    !supabaseUrl ? "SUPABASE_URL" : null,
    !serviceRoleKey ? "SUPABASE_SERVICE_ROLE_KEY" : null,
  ].filter((name): name is string => Boolean(name));

  if (!supabaseUrl || !serviceRoleKey) {
    throw new Error(
      `Supabase server configuration is missing: ${missingVariables.join(", ")}`,
    );
  }

  serverClient = createClient(supabaseUrl, serviceRoleKey, {
    auth: {
      autoRefreshToken: false,
      detectSessionInUrl: false,
      persistSession: false,
    },
  });

  return serverClient;
}
