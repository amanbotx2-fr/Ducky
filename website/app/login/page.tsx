import type { Metadata } from "next";
import { GitBranch, LockKeyhole } from "lucide-react";
import { redirect } from "next/navigation";
import { auth, signIn } from "../../auth";
import { BrandMark } from "../../components/BrandMark";
import { SectionContainer } from "../../components/SectionContainer";
import { isAllowedGitHubUsername } from "../../lib/auth/authorization";

export const metadata: Metadata = {
  title: "Admin Sign In — Ducky",
  description: "Sign in to Ducky's internal analytics.",
  robots: {
    follow: false,
    index: false,
  },
};

type LoginPageProps = {
  searchParams: Promise<{
    callbackUrl?: string | string[];
  }>;
};

function getSafeCallbackUrl(value: FormDataEntryValue | string[] | undefined) {
  const callbackUrl =
    typeof value === "string"
      ? value
      : Array.isArray(value)
        ? value[0]
        : undefined;

  if (
    callbackUrl?.startsWith("/admin") &&
    !callbackUrl.startsWith("//")
  ) {
    return callbackUrl;
  }

  return "/admin/analytics";
}

async function signInWithGitHub(formData: FormData) {
  "use server";

  await signIn("github", {
    redirectTo: getSafeCallbackUrl(formData.get("callbackUrl") ?? undefined),
  });
}

export default async function LoginPage({ searchParams }: LoginPageProps) {
  const params = await searchParams;
  const callbackUrl = getSafeCallbackUrl(params.callbackUrl);
  const session = await auth();

  if (session?.user) {
    if (isAllowedGitHubUsername(session.user.githubUsername)) {
      redirect(callbackUrl);
    }

    redirect("/admin/analytics");
  }

  return (
    <main className="min-h-screen bg-cream py-5 sm:py-8">
      <SectionContainer className="flex min-h-[calc(100vh-2.5rem)] flex-col">
        <header className="rounded-[22px] border-[3px] border-ink bg-cream px-4 py-4 shadow-brutal-window sm:px-6">
          <BrandMark compact />
        </header>

        <div className="flex flex-1 items-center justify-center py-12">
          <section
            aria-labelledby="admin-login-title"
            className="w-full max-w-[520px] rounded-[22px] border-[3px] border-ink bg-cream p-5 shadow-brutal-window sm:p-8"
          >
            <span className="grid size-14 place-items-center rounded-[15px] border-2 border-ink bg-yellow shadow-brutal-sm">
              <LockKeyhole
                aria-hidden="true"
                className="size-7"
                strokeWidth={2.4}
              />
            </span>
            <p className="mt-7 text-xs font-black uppercase tracking-[0.13em] text-orange">
              Internal access
            </p>
            <h1
              id="admin-login-title"
              className="mt-2 text-3xl font-black tracking-[-0.05em] sm:text-4xl"
            >
              Sign in to analytics
            </h1>
            <p className="mt-4 max-w-[440px] text-sm font-semibold leading-[1.7] text-ink/68 sm:text-base">
              Continue with an approved GitHub account to access Ducky&apos;s
              internal download dashboard.
            </p>

            <form action={signInWithGitHub} className="mt-8">
              <input type="hidden" name="callbackUrl" value={callbackUrl} />
              <button
                type="submit"
                className="flex min-h-14 w-full items-center justify-center gap-3 rounded-[12px] border-2 border-ink bg-orange px-5 text-sm font-black shadow-brutal transition-transform hover:-translate-y-0.5 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-orange/35"
              >
                <GitBranch
                  aria-hidden="true"
                  className="size-5"
                  strokeWidth={2.6}
                />
                Continue with GitHub
              </button>
            </form>

            <p className="mt-5 text-center text-xs font-semibold leading-relaxed text-ink/55">
              Authentication is handled securely by GitHub and Auth.js.
            </p>
          </section>
        </div>
      </SectionContainer>
    </main>
  );
}
