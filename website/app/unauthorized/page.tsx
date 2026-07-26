import type { Metadata } from "next";
import { ArrowLeft, ShieldX } from "lucide-react";
import Link from "next/link";
import { BrandMark } from "../../components/BrandMark";
import { SignOutButton } from "../../components/Auth/SignOutButton";
import { SectionContainer } from "../../components/SectionContainer";

export const metadata: Metadata = {
  title: "403 Unauthorized — Ducky",
  description: "This GitHub account cannot access Ducky analytics.",
  robots: {
    follow: false,
    index: false,
  },
};

export default function UnauthorizedPage() {
  return (
    <main className="min-h-screen bg-cream py-5 sm:py-8">
      <SectionContainer className="flex min-h-[calc(100vh-2.5rem)] flex-col">
        <header className="rounded-[22px] border-[3px] border-ink bg-cream px-4 py-4 shadow-brutal-window sm:px-6">
          <BrandMark compact />
        </header>

        <div className="flex flex-1 items-center justify-center py-12">
          <section
            aria-labelledby="unauthorized-title"
            className="w-full max-w-[560px] rounded-[22px] border-[3px] border-ink bg-cream p-5 shadow-brutal-window sm:p-8"
          >
            <span className="grid size-14 place-items-center rounded-[15px] border-2 border-ink bg-pink shadow-brutal-sm">
              <ShieldX
                aria-hidden="true"
                className="size-7"
                strokeWidth={2.4}
              />
            </span>
            <p className="mt-7 text-xs font-black uppercase tracking-[0.13em] text-orange">
              Error 403
            </p>
            <h1
              id="unauthorized-title"
              className="mt-2 text-3xl font-black tracking-[-0.05em] sm:text-4xl"
            >
              Unauthorized
            </h1>
            <p className="mt-4 max-w-[470px] text-sm font-semibold leading-[1.7] text-ink/68 sm:text-base">
              This GitHub account is signed in, but it is not approved to access
              Ducky&apos;s internal analytics.
            </p>

            <div className="mt-8 flex flex-col gap-3 sm:flex-row">
              <SignOutButton />
              <Link
                href="/"
                className="inline-flex min-h-11 items-center justify-center gap-2 rounded-[12px] border-2 border-ink bg-cream px-4 text-sm font-black shadow-brutal-sm transition-transform hover:-translate-y-0.5 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-orange/35"
              >
                <ArrowLeft
                  aria-hidden="true"
                  className="size-4"
                  strokeWidth={2.7}
                />
                Back to website
              </Link>
            </div>
          </section>
        </div>
      </SectionContainer>
    </main>
  );
}
