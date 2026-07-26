import type { ReactNode } from "react";
import { redirect } from "next/navigation";
import { auth } from "../../auth";
import { isAllowedGitHubUsername } from "../../lib/auth/authorization";

type AdminLayoutProps = {
  children: ReactNode;
};

export default async function AdminLayout({ children }: AdminLayoutProps) {
  const session = await auth();

  if (!session?.user) {
    redirect("/login?callbackUrl=/admin/analytics");
  }

  if (!isAllowedGitHubUsername(session.user.githubUsername)) {
    redirect("/unauthorized");
  }

  return children;
}
