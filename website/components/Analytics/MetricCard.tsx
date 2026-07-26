import type { LucideIcon } from "lucide-react";

type MetricCardProps = {
  label: string;
  value: number | null;
  icon: LucideIcon;
  iconColor: string;
  description: string;
};

const countFormatter = new Intl.NumberFormat("en-US");

export function MetricCard({
  label,
  value,
  icon: Icon,
  iconColor,
  description,
}: MetricCardProps) {
  return (
    <article className="flex min-h-[176px] min-w-0 flex-col rounded-[18px] border-2 border-ink bg-cream p-5 shadow-brutal-sm sm:min-h-[190px] sm:p-6">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0">
          <h3 className="text-sm font-black leading-tight tracking-[-0.02em] text-ink/70">
            {label}
          </h3>
          <p className="mt-3 text-[2.25rem] font-black leading-none tracking-[-0.06em] sm:text-[2.6rem]">
            {value === null ? "—" : countFormatter.format(value)}
          </p>
        </div>

        <span
          aria-hidden="true"
          className={`grid size-12 shrink-0 place-items-center rounded-[13px] border-2 border-ink shadow-brutal-sm ${iconColor}`}
        >
          <Icon className="size-6" strokeWidth={2.4} />
        </span>
      </div>

      <p className="mt-auto pt-5 text-xs font-semibold leading-relaxed text-ink/62">
        {description}
      </p>
    </article>
  );
}
