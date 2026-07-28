import {
  Code2,
  Feather,
  LockKeyhole,
  MonitorSmartphone,
  ShieldCheck,
  Sparkles,
  type LucideIcon,
} from "lucide-react";
import { motion } from "framer-motion";

export type Capability = {
  title: string;
  description: string;
  icon: LucideIcon;
  color: string;
};

const capabilities: Capability[] = [
  {
    title: "Lightweight",
    description: "Local-first and designed to stay out of the way.",
    icon: Feather,
    color: "bg-purple",
  },
  {
    title: "Cross-Platform",
    description: "Built for macOS, Windows, and Linux.",
    icon: MonitorSmartphone,
    color: "bg-mint",
  },
  {
    title: "Secure",
    description: "Credentials stay in native secure storage.",
    icon: ShieldCheck,
    color: "bg-yellow",
  },
  {
    title: "Open Source",
    description: "MIT licensed and built in the open.",
    icon: Code2,
    color: "bg-orange",
  },
  {
    title: "Privacy First",
    description: "Local-first design with no sign-up required.",
    icon: LockKeyhole,
    color: "bg-pink",
  },
  {
    title: "Always Improving",
    description: "Secure update foundations are included.",
    icon: Sparkles,
    color: "bg-blue",
  },
];

type CapabilityStripProps = {
  heading?: string;
  items?: Capability[];
  gridClassName?: string;
  className?: string;
};

export function CapabilityStrip({
  heading = "Built for productivity. Designed for everyone.",
  items = capabilities,
  gridClassName = "features-capability-grid",
  className = "mt-9",
}: CapabilityStripProps = {}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 24 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, amount: 0.22 }}
      transition={{ duration: 0.58, ease: [0.22, 1, 0.36, 1] }}
      className={`${className} rounded-[22px] border-[3px] border-ink bg-cream px-4 py-5 shadow-brutal-window sm:px-6`}
    >
      <h3 className="text-center text-sm font-black tracking-[-0.02em] sm:text-base">
        {heading}
      </h3>

      <div
        className={`${gridClassName} mt-5 grid grid-cols-1 gap-5 min-[480px]:grid-cols-2 lg:grid-cols-3`}
      >
        {items.map(
          ({ title, description, icon: Icon, color }, index) => (
            <motion.article
              key={title}
              initial={{ opacity: 0, y: 16 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, amount: 0.3 }}
              transition={{
                duration: 0.42,
                delay: 0.08 + index * 0.06,
                ease: [0.22, 1, 0.36, 1],
              }}
              className={`flex min-w-0 items-start gap-3 px-1 sm:px-2 min-[1400px]:px-4 ${
                index > 0
                  ? "min-[1400px]:border-l min-[1400px]:border-ink/55"
                  : ""
              }`}
            >
              <span
                className={`grid size-12 shrink-0 place-items-center rounded-[13px] border-2 border-ink shadow-brutal-sm ${color}`}
              >
                <Icon aria-hidden="true" className="size-6" strokeWidth={2.4} />
              </span>
              <span className="min-w-0">
                <h4 className="text-[0.78rem] font-black leading-tight tracking-[-0.02em] sm:text-[0.82rem]">
                  {title}
                </h4>
                <p className="mt-1.5 text-[0.67rem] font-semibold leading-[1.55] text-ink/76 sm:text-[0.7rem]">
                  {description}
                </p>
              </span>
            </motion.article>
          ),
        )}
      </div>
    </motion.div>
  );
}
