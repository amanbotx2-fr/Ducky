"use client";

import {
  Coffee,
  Download,
  Heart,
  Star,
  type LucideIcon,
} from "lucide-react";
import { motion } from "framer-motion";
import { supportLinks } from "../../lib/releaseAssets";

type FinalAction = {
  label: string;
  href?: string;
  icon: LucideIcon;
  className: string;
  external?: boolean;
};

const finalActions: FinalAction[] = [
  {
    label: "Download Ducky",
    href: "#download",
    icon: Download,
    className: "bg-orange",
  },
  {
    label: "Star on GitHub",
    href: supportLinks.repository,
    icon: Star,
    className: "bg-cream",
    external: true,
  },
  {
    label: "Buy Me a Coffee",
    href: process.env.NEXT_PUBLIC_BUY_ME_A_COFFEE_URL,
    icon: Coffee,
    className: "bg-yellow",
    external: true,
  },
];

function CTAAction({ action }: { action: FinalAction }) {
  const { label, href, icon: Icon, className, external } = action;
  const classes = `flex min-h-12 w-full items-center justify-center gap-2 rounded-[12px] border-2 border-ink px-5 text-sm font-black shadow-brutal-sm transition-[transform,box-shadow] duration-300 ${className} ${
    href
      ? "hover:-translate-y-1 hover:shadow-brutal focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-orange/35"
      : "cursor-not-allowed opacity-60"
  }`;

  if (!href) {
    return (
      <span
        aria-disabled="true"
        title={`${label} link is not configured yet`}
        className={classes}
      >
        {label}
        <Icon aria-hidden="true" className="size-5" strokeWidth={2.6} />
      </span>
    );
  }

  return (
    <a
      href={href}
      target={external ? "_blank" : undefined}
      rel={external ? "noreferrer" : undefined}
      aria-label={external ? `${label} (opens in a new tab)` : label}
      className={classes}
    >
      {label}
      <Icon
        aria-hidden="true"
        className={`size-5 ${label === "Star on GitHub" ? "fill-yellow" : ""}`}
        strokeWidth={2.6}
      />
    </a>
  );
}

export function FinalCTA() {
  return (
    <motion.aside
      initial={{ opacity: 0, y: 22 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, amount: 0.2 }}
      transition={{ duration: 0.56, ease: [0.22, 1, 0.36, 1] }}
      aria-labelledby="final-cta-title"
      className="mt-8 grid items-center gap-6 rounded-[22px] border-[3px] border-ink bg-yellow/75 p-5 shadow-brutal-window sm:p-6 lg:grid-cols-[minmax(0,1fr)_minmax(620px,1.25fr)]"
    >
      <div className="flex min-w-0 items-center gap-4">
        <span className="pixel-bubble relative grid size-[74px] shrink-0 place-items-center bg-cream shadow-brutal-sm">
          <Heart
            aria-hidden="true"
            className="size-8 fill-orange"
            strokeWidth={2.6}
          />
        </span>
        <h3
          id="final-cta-title"
          className="text-[clamp(1.65rem,3vw,2.45rem)] font-black leading-[1.02] tracking-[-0.05em]"
        >
          Ready to bring Ducky <span className="text-orange">home?</span>
        </h3>
      </div>

      <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
        {finalActions.map((action) => (
          <CTAAction key={action.label} action={action} />
        ))}
      </div>
    </motion.aside>
  );
}
