import { Apple, Download, MonitorDown } from "lucide-react";
import { motion } from "framer-motion";

const releaseUrl = "https://github.com/amanbotx2-fr/Ducky/releases/latest";

const downloads = [
  {
    label: "Download for macOS",
    shortLabel: "macOS",
    icon: Apple,
    tone: "bg-orange",
  },
  {
    label: "Download for Windows",
    shortLabel: "Windows",
    icon: MonitorDown,
    tone: "bg-cream",
  },
  {
    label: "Download for Linux",
    shortLabel: "Linux",
    icon: Download,
    tone: "bg-cream",
  },
];

export function HeroButtons() {
  return (
    <div
      className="grid w-full grid-cols-1 gap-3 sm:grid-cols-3 lg:max-w-[690px]"
    >
      {downloads.map(({ label, shortLabel, icon: Icon, tone }) => (
        <motion.a
          key={label}
          href={releaseUrl}
          target="_blank"
          rel="noreferrer"
          whileHover={{ y: -5 }}
          whileTap={{ y: 1 }}
          transition={{ type: "spring", stiffness: 380, damping: 24 }}
          className={`flex h-[62px] min-w-0 items-center justify-center gap-2.5 rounded-[13px] border-2 border-ink px-3 text-center text-sm font-black tracking-[-0.02em] shadow-brutal focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-orange/30 ${tone}`}
          aria-label={label}
        >
          <Icon aria-hidden="true" className="size-6 shrink-0" strokeWidth={2.6} />
          <span className="hidden min-[1380px]:inline">{label}</span>
          <span className="min-[1380px]:hidden">{shortLabel}</span>
        </motion.a>
      ))}
    </div>
  );
}
