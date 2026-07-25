import {
  BellRing,
  BotMessageSquare,
  Clock3,
  NotebookPen,
  Pin,
  Settings2,
  type LucideIcon,
} from "lucide-react";
import { motion } from "framer-motion";

type Feature = {
  title: string;
  description: string;
  icon: LucideIcon;
  color: string;
};

const features: Feature[] = [
  {
    title: "AI Chat",
    description: "OpenAI, Gemini, Grok, Ollama and more.",
    icon: BotMessageSquare,
    color: "bg-purple",
  },
  {
    title: "Pomodoro Timer",
    description: "Custom focus sessions and gentle break cues.",
    icon: Clock3,
    color: "bg-yellow",
  },
  {
    title: "Sticky Notes",
    description: "Quick notes that stay close to your work.",
    icon: NotebookPen,
    color: "bg-mint",
  },
  {
    title: "Smart Reminders",
    description: "One-time or recurring reminders.",
    icon: Pin,
    color: "bg-pink",
  },
  {
    title: "Notification Sounds",
    description:
      "Beautiful built-in notification sounds for reminders and focus sessions.",
    icon: BellRing,
    color: "bg-orange",
  },
  {
    title: "And More",
    description: "Tray controls, themes and thoughtful extras.",
    icon: Settings2,
    color: "bg-orange",
  },
];

export function FeatureStrip() {
  return (
    <motion.div
      initial={false}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.6, delay: 0.62, ease: [0.22, 1, 0.36, 1] }}
      className="feature-strip-enter relative z-20 mt-7 rounded-[22px] border-[3px] border-ink bg-cream px-4 py-5 shadow-brutal-window sm:px-5 xl:mt-[-4px] xl:px-6"
    >
      <div className="grid grid-cols-1 gap-x-3 gap-y-6 min-[460px]:grid-cols-2 sm:grid-cols-3 min-[1400px]:grid-cols-6 min-[1400px]:gap-0">
        {features.map(({ title, description, icon: Icon, color }, index) => (
          <article
            key={title}
            className={`flex min-w-0 items-start gap-3 px-1 sm:px-2 min-[1400px]:px-4 ${
              index > 0
                ? "min-[1400px]:border-l min-[1400px]:border-ink/55"
                : ""
            }`}
          >
            <span
              className={`grid size-12 shrink-0 place-items-center rounded-[13px] border-2 border-ink shadow-brutal-sm sm:size-[54px] ${color}`}
            >
              <Icon aria-hidden="true" className="size-6 sm:size-7" strokeWidth={2.4} />
            </span>
            <span className="min-w-0">
              <h2 className="text-[0.79rem] font-black leading-tight tracking-[-0.025em] sm:text-sm">
                {title}
              </h2>
              <p className="mt-1.5 text-[0.68rem] font-semibold leading-relaxed text-ink/78 sm:text-xs">
                {description}
              </p>
            </span>
          </article>
        ))}
      </div>
    </motion.div>
  );
}
