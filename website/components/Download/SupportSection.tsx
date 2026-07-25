import {
  ArrowRight,
  BookOpenText,
  Bug,
  Heart,
  MessageCircle,
  Star,
  type LucideIcon,
} from "lucide-react";
import { motion } from "framer-motion";
import { supportLinks } from "../../lib/releaseAssets";

type SupportItem = {
  title: string;
  description: string;
  href: string;
  icon: LucideIcon;
  color: string;
};

const supportItems: SupportItem[] = [
  {
    title: "Documentation",
    description: "Step-by-step guides.",
    href: supportLinks.documentation,
    icon: BookOpenText,
    color: "bg-purple",
  },
  {
    title: "Community",
    description: "Join discussions.",
    href: supportLinks.community,
    icon: MessageCircle,
    color: "bg-mint",
  },
  {
    title: "Report Issue",
    description: "Open a GitHub Issue.",
    href: supportLinks.issue,
    icon: Bug,
    color: "bg-orange",
  },
  {
    title: "GitHub",
    description: "Star Ducky.",
    href: supportLinks.repository,
    icon: Star,
    color: "bg-pink",
  },
];

export function SupportSection() {
  return (
    <motion.section
      initial={{ opacity: 0, y: 24 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, amount: 0.15 }}
      transition={{ duration: 0.56, ease: [0.22, 1, 0.36, 1] }}
      aria-labelledby="support-title"
      className="mt-12 rounded-[22px] border-[3px] border-ink bg-cream p-4 shadow-brutal-window sm:p-6"
    >
      <div className="flex items-start gap-3">
        <span className="grid size-11 shrink-0 place-items-center rounded-full border-2 border-ink bg-yellow shadow-brutal-sm">
          <Heart aria-hidden="true" className="size-6 fill-pink" strokeWidth={2.4} />
        </span>
        <div>
          <h3
            id="support-title"
            className="text-2xl font-black tracking-[-0.04em] sm:text-3xl"
          >
            Still need help?
          </h3>
          <p className="mt-1 text-sm font-semibold text-ink/70">
            We&apos;re here if you get stuck.
          </p>
        </div>
      </div>

      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
        {supportItems.map(
          ({ title, description, href, icon: Icon, color }, index) => (
            <motion.a
              key={title}
              href={href}
              target="_blank"
              rel="noreferrer"
              aria-label={`${title}: ${description} (opens in a new tab)`}
              initial={{ opacity: 0, y: 16 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, amount: 0.3 }}
              whileHover={{ y: -6, rotate: index % 2 === 0 ? -0.7 : 0.7 }}
              transition={{
                duration: 0.44,
                delay: index * 0.055,
                ease: [0.22, 1, 0.36, 1],
              }}
              className="group flex min-h-[132px] min-w-0 items-start gap-3 rounded-[16px] border-2 border-ink bg-cream p-4 shadow-brutal outline-none focus-visible:ring-4 focus-visible:ring-orange/35"
            >
              <span
                className={`grid size-12 shrink-0 place-items-center rounded-[12px] border-2 border-ink shadow-brutal-sm ${color}`}
              >
                <Icon aria-hidden="true" className="size-6" strokeWidth={2.4} />
              </span>
              <span className="flex min-h-[96px] min-w-0 flex-1 flex-col">
                <span className="text-sm font-black">{title}</span>
                <span className="mt-1.5 text-xs font-semibold leading-relaxed text-ink/70">
                  {description}
                </span>
                <ArrowRight
                  aria-hidden="true"
                  className="mt-auto size-5 self-end transition-transform duration-300 group-hover:translate-x-1.5"
                  strokeWidth={2.6}
                />
              </span>
            </motion.a>
          ),
        )}
      </div>
    </motion.section>
  );
}
