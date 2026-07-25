"use client";

import {
  Coffee,
  ExternalLink,
  Heart,
  IndianRupee,
  type LucideIcon,
} from "lucide-react";
import { motion } from "framer-motion";

type SupportCardData = {
  title: string;
  description: string;
  action: string;
  href?: string;
  icon: LucideIcon;
  color: string;
  primary?: boolean;
};

const supportCards: SupportCardData[] = [
  {
    title: "Buy Me a Coffee",
    description: "One-time or monthly support to keep Ducky growing.",
    action: "Buy Me a Coffee",
    href: process.env.NEXT_PUBLIC_BUY_ME_A_COFFEE_URL,
    icon: Coffee,
    color: "bg-yellow",
    primary: true,
  },
  {
    title: "GitHub Sponsors",
    description: "Sponsor development and get recognition on GitHub.",
    action: "Sponsor on GitHub",
    href: process.env.NEXT_PUBLIC_GITHUB_SPONSORS_URL,
    icon: Heart,
    color: "bg-pink",
  },
  {
    title: "Support via UPI",
    description: "Prefer direct support? Send any amount via UPI.",
    action: "View UPI Details",
    href: process.env.NEXT_PUBLIC_UPI_SUPPORT_URL,
    icon: IndianRupee,
    color: "bg-mint",
  },
];

function SupportAction({
  title,
  action,
  href,
  primary = false,
}: Pick<SupportCardData, "title" | "action" | "href" | "primary">) {
  const className = `mt-auto flex min-h-12 w-full items-center justify-center gap-2 rounded-[12px] border-2 border-ink px-4 text-sm font-black shadow-brutal-sm transition-[transform,box-shadow] duration-300 ${
    primary ? "bg-orange" : "bg-cream"
  } ${
    href
      ? "group-hover/action:-translate-y-0.5 group-hover/action:shadow-brutal focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-orange/35"
      : "cursor-not-allowed opacity-60"
  }`;

  if (!href) {
    return (
      <span
        aria-disabled="true"
        title={`${title} link is not configured yet`}
        className={className}
      >
        {action}
        <ExternalLink aria-hidden="true" className="size-4" strokeWidth={2.5} />
      </span>
    );
  }

  return (
    <a
      href={href}
      target="_blank"
      rel="noreferrer"
      aria-label={`${action} (opens in a new tab)`}
      className={`group/action ${className}`}
    >
      {action}
      <ExternalLink aria-hidden="true" className="size-4" strokeWidth={2.5} />
    </a>
  );
}

export function SupportCards() {
  return (
    <section
      aria-labelledby="support-options-title"
      className="mt-11 rounded-[22px] border-[3px] border-ink bg-cream p-4 shadow-brutal-window sm:p-6 lg:p-7"
    >
      <h3
        id="support-options-title"
        className="text-center text-xl font-black tracking-[-0.035em] sm:text-2xl"
      >
        Choose a way to support Ducky{" "}
        <span aria-hidden="true" className="text-pink">
          ❤️
        </span>
      </h3>

      <div className="mt-7 grid grid-cols-1 gap-5 sm:grid-cols-2 xl:grid-cols-4">
        {supportCards.map(
          (
            { title, description, action, href, icon: Icon, color, primary },
            index,
          ) => (
            <motion.article
              key={title}
              initial={{ opacity: 0, y: 22 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, amount: 0.25 }}
              whileHover={{ y: -6, rotate: index % 2 === 0 ? -0.6 : 0.6 }}
              transition={{
                duration: 0.48,
                delay: index * 0.07,
                ease: [0.22, 1, 0.36, 1],
              }}
              className="flex min-h-[254px] min-w-0 flex-col rounded-[18px] border-2 border-ink bg-cream p-5 shadow-brutal"
            >
              <div className="flex items-start gap-4">
                <span
                  className={`grid size-16 shrink-0 place-items-center rounded-[15px] border-[3px] border-ink shadow-brutal-sm ${color}`}
                >
                  <Icon aria-hidden="true" className="size-8" strokeWidth={2.4} />
                </span>
                <div className="min-w-0 pt-1">
                  <h4 className="text-lg font-black leading-tight tracking-[-0.035em]">
                    {title}
                  </h4>
                  <p className="mt-2 text-sm font-semibold leading-[1.6] text-ink/74">
                    {description}
                  </p>
                </div>
              </div>

              <SupportAction
                title={title}
                action={action}
                href={href}
                primary={primary}
              />
            </motion.article>
          ),
        )}

        <motion.aside
          initial={{ opacity: 0, y: 22 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, amount: 0.25 }}
          transition={{
            duration: 0.48,
            delay: supportCards.length * 0.07,
            ease: [0.22, 1, 0.36, 1],
          }}
          className="flex min-h-[254px] flex-col items-center justify-center rounded-[18px] border-2 border-[#e5bd54] bg-[linear-gradient(145deg,#fff9ef_0%,#fff0c9_100%)] p-7 text-center"
        >
          <p className="max-w-[230px] text-base font-bold leading-[1.75] text-ink/82">
            Every bit of support goes into making Ducky better for everyone.
          </p>
          <Heart
            aria-hidden="true"
            className="mt-7 size-10 fill-yellow text-ink drop-shadow-[2px_2px_0_#111]"
            strokeWidth={2.1}
          />
        </motion.aside>
      </div>
    </section>
  );
}
