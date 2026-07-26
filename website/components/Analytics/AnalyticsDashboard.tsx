import {
  ArrowLeft,
  CalendarDays,
  CalendarRange,
  Database,
  Download,
} from "lucide-react";
import Link from "next/link";
import type { DownloadAnalyticsOverview } from "../../lib/analytics/types";
import { BrandMark } from "../BrandMark";
import { SignOutButton } from "../Auth/SignOutButton";
import { SectionContainer } from "../SectionContainer";
import { MetricCard } from "./MetricCard";
import { PlatformBreakdown } from "./PlatformBreakdown";
import { ReleaseBreakdown } from "./ReleaseBreakdown";

type AnalyticsDashboardProps = {
  data: DownloadAnalyticsOverview;
  isAvailable: boolean;
};

export function AnalyticsDashboard({
  data,
  isAvailable,
}: AnalyticsDashboardProps) {
  const metricValue = (value: number) => (isAvailable ? value : null);

  return (
    <main
      id="top"
      className="relative min-h-screen overflow-hidden bg-cream py-5 sm:py-7 lg:py-9"
    >
      <span
        aria-hidden="true"
        className="absolute -left-4 top-52 hidden size-9 rotate-12 rounded-lg border-[3px] border-ink bg-purple shadow-brutal sm:block"
      />
      <span
        aria-hidden="true"
        className="absolute -right-3 top-[38rem] hidden size-8 -rotate-12 rounded-full border-[3px] border-ink bg-mint shadow-brutal sm:block"
      />

      <SectionContainer className="relative">
        <header className="flex flex-wrap items-center justify-between gap-4 rounded-[22px] border-[3px] border-ink bg-cream px-4 py-4 shadow-brutal-window sm:px-6">
          <BrandMark compact />
          <div className="flex flex-wrap items-center gap-3">
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
            <SignOutButton />
          </div>
        </header>

        <section
          aria-labelledby="analytics-page-title"
          className="pb-9 pt-12 sm:pb-11 sm:pt-16 lg:pb-12"
        >
          <div className="inline-flex min-h-10 items-center gap-2 rounded-[10px] border-2 border-ink bg-yellow px-3.5 py-2 text-xs font-black uppercase tracking-[0.08em] shadow-brutal-sm">
            <Database aria-hidden="true" className="size-4" strokeWidth={2.6} />
            Internal analytics
          </div>
          <h1
            id="analytics-page-title"
            className="mt-6 max-w-[840px] text-[2.8rem] font-black leading-[0.98] tracking-[-0.065em] min-[480px]:text-[3.5rem] sm:text-[4.25rem] lg:text-[5.2rem]"
          >
            Download analytics.
            <span className="block text-orange">At a glance.</span>
          </h1>
          <p className="mt-6 max-w-[660px] text-base font-semibold leading-[1.75] text-ink/68 sm:text-lg">
            Server-side totals for tracked Ducky download redirects, grouped by
            time period, platform, and release.
          </p>
          <p className="mt-4 text-xs font-black uppercase tracking-[0.1em] text-ink/48">
            Calendar periods use UTC
          </p>
        </section>

        {!isAvailable ? (
          <aside
            role="status"
            className="mb-7 flex items-start gap-3 rounded-[16px] border-2 border-ink bg-yellow px-4 py-4 shadow-brutal-sm sm:px-5"
          >
            <Database
              aria-hidden="true"
              className="mt-0.5 size-5 shrink-0"
              strokeWidth={2.5}
            />
            <div>
              <h2 className="text-sm font-black">Analytics temporarily unavailable</h2>
              <p className="mt-1 text-xs font-semibold leading-relaxed text-ink/70 sm:text-sm">
                The overview could not reach its aggregate data source. The
                public download routes continue to work normally.
              </p>
            </div>
          </aside>
        ) : null}

        <section aria-labelledby="overview-title">
          <div className="mb-5 flex items-center gap-3">
            <span className="h-1 w-9 rounded-full bg-orange" />
            <h2
              id="overview-title"
              className="text-2xl font-black tracking-[-0.045em] sm:text-[1.75rem]"
            >
              Overview
            </h2>
          </div>

          <div className="grid grid-cols-1 gap-4 min-[480px]:grid-cols-2 xl:grid-cols-4">
            <MetricCard
              label="Total Downloads"
              value={metricValue(data.totalDownloads)}
              icon={Download}
              iconColor="bg-orange"
              description="All tracked download redirects."
            />
            <MetricCard
              label="Downloads Today"
              value={metricValue(data.downloadsToday)}
              icon={CalendarDays}
              iconColor="bg-yellow"
              description="Since 00:00 UTC today."
            />
            <MetricCard
              label="Downloads This Week"
              value={metricValue(data.downloadsThisWeek)}
              icon={CalendarRange}
              iconColor="bg-mint"
              description="Since Monday at 00:00 UTC."
            />
            <MetricCard
              label="Downloads This Month"
              value={metricValue(data.downloadsThisMonth)}
              icon={Database}
              iconColor="bg-purple"
              description="Since the first day of this month."
            />
          </div>
        </section>

        <div className="mt-7 grid min-w-0 grid-cols-1 gap-7 lg:grid-cols-[minmax(280px,0.78fr)_minmax(0,1.45fr)]">
          <PlatformBreakdown
            platforms={data.platforms}
            isAvailable={isAvailable}
          />
          <ReleaseBreakdown
            releases={data.releases}
            isAvailable={isAvailable}
          />
        </div>

        <footer className="mt-9 border-t-2 border-ink/20 py-6 text-center text-xs font-bold text-ink/55">
          Ducky internal download analytics · No visitor data included
        </footer>
      </SectionContainer>
    </main>
  );
}
