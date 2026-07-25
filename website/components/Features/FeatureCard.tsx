import { ArrowRight, type LucideIcon } from "lucide-react";
import { motion } from "framer-motion";
import { supportLinks } from "../../lib/siteLinks";

export type FeatureAccent =
  | "purple"
  | "yellow"
  | "mint"
  | "pink"
  | "orange"
  | "blue";

export type FeatureCardData = {
  title: string;
  description: string;
  tag: string;
  icon: LucideIcon;
  accent: FeatureAccent;
  rotation: number;
};

type FeatureCardProps = FeatureCardData & {
  index: number;
};

const accentStyles: Record<
  FeatureAccent,
  { icon: string; tag: string }
> = {
  purple: {
    icon: "bg-purple",
    tag: "border-[#c69aff] bg-[#eadcff]",
  },
  yellow: {
    icon: "bg-yellow",
    tag: "border-[#e9bd2a] bg-[#fff0ae]",
  },
  mint: {
    icon: "bg-mint",
    tag: "border-[#45bcb4] bg-[#c9f1ee]",
  },
  pink: {
    icon: "bg-pink",
    tag: "border-[#f16c91] bg-[#ffd5e1]",
  },
  orange: {
    icon: "bg-orange",
    tag: "border-[#f25a2b] bg-[#ffd8ca]",
  },
  blue: {
    icon: "bg-blue",
    tag: "border-[#3da3d9] bg-[#d6effc]",
  },
};

export function FeatureCard({
  title,
  description,
  tag,
  icon: Icon,
  accent,
  rotation,
  index,
}: FeatureCardProps) {
  const styles = accentStyles[accent];
  const hoverRotation = rotation <= 0 ? -1.2 : 1.2;

  return (
    <motion.a
      href={supportLinks.features}
      target="_blank"
      rel="noreferrer"
      aria-label={`${title}: view the feature overview on GitHub (opens in a new tab)`}
      initial={{ opacity: 0, y: 28, rotate: rotation }}
      whileInView={{ opacity: 1, y: 0, rotate: rotation }}
      viewport={{ once: true, amount: 0.22 }}
      whileHover={{ y: -8, rotate: hoverRotation }}
      whileFocus={{ y: -6, rotate: hoverRotation }}
      transition={{
        duration: 0.5,
        delay: index * 0.07,
        ease: [0.22, 1, 0.36, 1],
      }}
      className="group relative flex min-h-[236px] min-w-0 flex-col rounded-[19px] border-[3px] border-ink bg-cream p-4 shadow-brutal-lg outline-none focus-visible:ring-4 focus-visible:ring-orange/35 min-[360px]:min-h-[252px] min-[360px]:p-5 sm:min-h-[264px]"
    >
      <div className="flex min-w-0 items-start gap-4">
        <motion.span
          aria-hidden="true"
          className={`grid size-[60px] shrink-0 place-items-center rounded-[14px] border-[3px] border-ink shadow-brutal-sm transition-transform duration-300 group-hover:-rotate-2 group-focus-visible:-rotate-2 min-[360px]:size-[68px] min-[360px]:rounded-[16px] sm:size-[74px] ${styles.icon}`}
        >
          <Icon
            className="size-7 min-[360px]:size-8 sm:size-9"
            strokeWidth={2.35}
          />
        </motion.span>

        <span className="min-w-0">
          <h3 className="text-base font-black leading-tight tracking-[-0.035em] min-[360px]:text-lg">
            {title}
          </h3>
          <p className="mt-2 text-[0.76rem] font-semibold leading-[1.65] text-ink/80 min-[360px]:text-[0.78rem] sm:text-[0.82rem]">
            {description}
          </p>
        </span>
      </div>

      <div className="mt-auto flex items-end justify-between gap-4 pt-5">
        <span
          className={`inline-flex min-h-7 items-center rounded-md border px-2.5 py-1 text-[0.68rem] font-black leading-none ${styles.tag}`}
        >
          {tag}
        </span>
        <motion.span
          aria-hidden="true"
          className="grid size-9 shrink-0 place-items-center rounded-full transition-transform duration-300 group-hover:translate-x-1.5 group-focus-visible:translate-x-1.5"
        >
          <ArrowRight className="size-6" strokeWidth={2.6} />
        </motion.span>
      </div>
    </motion.a>
  );
}
