import { AppWindow, TerminalSquare } from "lucide-react";
import type { DownloadAnalyticsOverview } from "../../lib/analytics/types";
import { AppleLogoIcon } from "../icons/AppleLogoIcon";

type PlatformBreakdownProps = {
  platforms: DownloadAnalyticsOverview["platforms"];
  isAvailable: boolean;
};

const countFormatter = new Intl.NumberFormat("en-US");

const platforms = [
  {
    key: "mac",
    label: "macOS",
    icon: AppleLogoIcon,
    iconColor: "bg-purple",
  },
  {
    key: "windows",
    label: "Windows",
    icon: AppWindow,
    iconColor: "bg-mint",
  },
  {
    key: "linux",
    label: "Linux",
    icon: TerminalSquare,
    iconColor: "bg-yellow",
  },
] as const;

export function PlatformBreakdown({
  platforms: platformCounts,
  isAvailable,
}: PlatformBreakdownProps) {
  return (
    <section
      aria-labelledby="platform-breakdown-title"
      className="rounded-[22px] border-[3px] border-ink bg-cream p-4 shadow-brutal-window sm:p-6"
    >
      <div>
        <p className="text-xs font-black uppercase tracking-[0.13em] text-orange">
          By operating system
        </p>
        <h2
          id="platform-breakdown-title"
          className="mt-2 text-2xl font-black tracking-[-0.045em] sm:text-[1.75rem]"
        >
          Platform Breakdown
        </h2>
      </div>

      <div className="mt-6 space-y-3">
        {platforms.map(({ key, label, icon: Icon, iconColor }) => (
          <article
            key={key}
            className="flex min-h-[76px] items-center gap-4 rounded-[15px] border-2 border-ink bg-cream px-4 py-3 shadow-brutal-sm"
          >
            <span
              aria-hidden="true"
              className={`grid size-11 shrink-0 place-items-center rounded-[12px] border-2 border-ink ${iconColor}`}
            >
              <Icon className="size-[22px]" strokeWidth={2.35} />
            </span>
            <h3 className="min-w-0 flex-1 text-base font-black tracking-[-0.025em]">
              {label}
            </h3>
            <p
              aria-label={`${label} downloads`}
              className="text-xl font-black tabular-nums tracking-[-0.045em]"
            >
              {isAvailable
                ? countFormatter.format(platformCounts[key])
                : "—"}
            </p>
          </article>
        ))}
      </div>
    </section>
  );
}
