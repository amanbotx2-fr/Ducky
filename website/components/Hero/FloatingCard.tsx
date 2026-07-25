import type { LucideIcon } from "lucide-react";
import { ArrowRight } from "lucide-react";
import { motion } from "framer-motion";

type FloatingCardProps = {
  title: string;
  description: string;
  icon: LucideIcon;
  color: "purple" | "mint" | "yellow";
  className?: string;
  delay?: number;
  rotate?: number;
};

const colors = {
  purple: "bg-purple",
  mint: "bg-mint",
  yellow: "bg-yellow",
};

export function FloatingCard({
  title,
  description,
  icon: Icon,
  color,
  className = "",
  delay = 0,
  rotate = 0,
}: FloatingCardProps) {
  return (
    <motion.article
      initial={false}
      animate={{
        opacity: 1,
        x: 0,
        scale: 1,
        rotate,
        y: [0, -6, 0],
      }}
      transition={{
        opacity: { duration: 0.4, delay },
        x: { duration: 0.45, delay },
        scale: { duration: 0.45, delay },
        rotate: { duration: 0.45, delay },
        y: {
          duration: 4.8,
          delay: delay + 0.7,
          repeat: Infinity,
          ease: "easeInOut",
        },
      }}
      className={`rounded-[17px] border-2 border-ink p-4 shadow-brutal-lg ${colors[color]} ${className}`}
    >
      <div className="flex items-center gap-2.5">
        <span className="grid size-9 shrink-0 place-items-center rounded-[10px] border-2 border-ink bg-cream">
          <Icon aria-hidden="true" className="size-5" strokeWidth={2.5} />
        </span>
        <h3 className="text-base font-black tracking-[-0.035em] sm:text-lg">
          {title}
        </h3>
      </div>
      <div className="mt-3 flex items-end justify-between gap-4">
        <p className="max-w-[16ch] text-xs font-semibold leading-relaxed sm:text-sm">
          {description}
        </p>
        <ArrowRight
          aria-hidden="true"
          className="size-6 shrink-0"
          strokeWidth={2.6}
        />
      </div>
    </motion.article>
  );
}
