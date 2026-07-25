import { ShieldCheck } from "lucide-react";

type BadgeProps = {
  children: React.ReactNode;
  compact?: boolean;
};

export function Badge({ children, compact = false }: BadgeProps) {
  return (
    <span
      className={`inline-flex items-center rounded-xl border-2 border-ink bg-yellow font-extrabold uppercase tracking-[-0.02em] shadow-brutal-sm ${
        compact
          ? "gap-2 px-3 py-2 text-[0.69rem]"
          : "gap-2.5 px-4 py-2.5 text-xs sm:px-5 sm:text-sm"
      }`}
    >
      <ShieldCheck aria-hidden="true" className="size-[1.15em]" strokeWidth={2.8} />
      {children}
    </span>
  );
}
