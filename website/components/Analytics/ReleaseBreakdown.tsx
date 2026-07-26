import { PackageOpen } from "lucide-react";
import type { ReleaseDownloadCount } from "../../lib/analytics/types";

type ReleaseBreakdownProps = {
  releases: ReleaseDownloadCount[];
  isAvailable: boolean;
};

const countFormatter = new Intl.NumberFormat("en-US");

export function ReleaseBreakdown({
  releases,
  isAvailable,
}: ReleaseBreakdownProps) {
  const isEmpty = isAvailable && releases.length === 0;

  return (
    <section
      aria-labelledby="release-breakdown-title"
      className="min-w-0 rounded-[22px] border-[3px] border-ink bg-cream p-4 shadow-brutal-window sm:p-6"
    >
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <p className="text-xs font-black uppercase tracking-[0.13em] text-orange">
            By resolved GitHub release
          </p>
          <h2
            id="release-breakdown-title"
            className="mt-2 text-2xl font-black tracking-[-0.045em] sm:text-[1.75rem]"
          >
            Release Breakdown
          </h2>
        </div>
        {isAvailable && releases.length > 0 ? (
          <span className="rounded-lg border-2 border-ink bg-yellow px-3 py-1.5 text-xs font-black shadow-brutal-sm">
            {releases.length} {releases.length === 1 ? "release" : "releases"}
          </span>
        ) : null}
      </div>

      {isAvailable && releases.length > 0 ? (
        <div className="mt-6 overflow-hidden rounded-[15px] border-2 border-ink">
          <table className="w-full border-collapse text-left">
            <thead className="bg-yellow">
              <tr>
                <th
                  scope="col"
                  className="px-4 py-3 text-xs font-black uppercase tracking-[0.1em] sm:px-5"
                >
                  Release
                </th>
                <th
                  scope="col"
                  className="px-4 py-3 text-right text-xs font-black uppercase tracking-[0.1em] sm:px-5"
                >
                  Downloads
                </th>
              </tr>
            </thead>
            <tbody>
              {releases.map((release, index) => (
                <tr
                  key={release.version}
                  className={
                    index > 0 ? "border-t-2 border-ink" : undefined
                  }
                >
                  <th
                    scope="row"
                    className="max-w-0 px-4 py-4 text-sm font-black sm:px-5 sm:text-base"
                  >
                    <span className="block truncate">{release.version}</span>
                  </th>
                  <td className="px-4 py-4 text-right text-base font-black tabular-nums sm:px-5">
                    {countFormatter.format(release.downloads)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <div
          className="mt-6 flex min-h-[228px] flex-col items-center justify-center rounded-[15px] border-2 border-dashed border-ink/45 bg-[#fff6e5] px-5 py-8 text-center"
          role="status"
        >
          <span
            aria-hidden="true"
            className="grid size-14 place-items-center rounded-[15px] border-2 border-ink bg-yellow shadow-brutal-sm"
          >
            <PackageOpen className="size-7" strokeWidth={2.25} />
          </span>
          <h3 className="mt-5 text-lg font-black tracking-[-0.03em]">
            {isEmpty ? "No download events yet" : "Release data unavailable"}
          </h3>
          <p className="mt-2 max-w-[390px] text-sm font-semibold leading-relaxed text-ink/65">
            {isEmpty
              ? "The first tracked download redirect will appear here automatically."
              : "The dashboard could not read release totals. No download behavior is affected."}
          </p>
        </div>
      )}
    </section>
  );
}
