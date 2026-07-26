export const allowedGitHubUsernames = ["amanbotx2-fr"] as const;

const normalizedAllowedUsernames = new Set(
  allowedGitHubUsernames.map((username) => username.toLowerCase()),
);

export function isAllowedGitHubUsername(
  username: string | null | undefined,
): boolean {
  return Boolean(
    username && normalizedAllowedUsernames.has(username.toLowerCase()),
  );
}
