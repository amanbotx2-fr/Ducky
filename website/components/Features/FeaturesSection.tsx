"use client";

import {
  BellRing,
  BotMessageSquare,
  Circle,
  Diamond,
  MonitorCog,
  Search,
  Star,
  StickyNote,
  Timer,
} from "lucide-react";
import { motion, MotionConfig } from "framer-motion";
import { SectionContainer } from "../SectionContainer";
import { CapabilityStrip } from "./CapabilityStrip";
import {
  FeatureCard,
  type FeatureCardData,
} from "./FeatureCard";
import { FeaturesMascotPanel } from "./FeaturesMascotPanel";

const features: FeatureCardData[] = [
  {
    title: "AI Chat",
    description:
      "Chat with OpenAI, Gemini, Groq, Ollama, and OpenAI-compatible providers directly from your desktop.",
    tag: "Multiple Models",
    icon: BotMessageSquare,
    accent: "purple",
    rotation: -0.35,
  },
  {
    title: "Pomodoro Timer",
    description:
      "Run beautiful Pomodoro sessions, play notification sounds, and stay productive with break reminders.",
    tag: "Focus Better",
    icon: Timer,
    accent: "yellow",
    rotation: 0.28,
  },
  {
    title: "Sticky Notes",
    description:
      "Create lightweight sticky notes that stay available while you work.",
    tag: "Always Visible",
    icon: StickyNote,
    accent: "mint",
    rotation: -0.2,
  },
  {
    title: "Smart Reminders",
    description:
      "One-time reminders, recurring reminders, hydration reminders, and desktop notifications.",
    tag: "Never Forget",
    icon: BellRing,
    accent: "pink",
    rotation: 0.24,
  },
  {
    title: "AI Model Explorer",
    description:
      "Search, favorite, and switch between supported AI models without memorizing IDs.",
    tag: "Hundreds of Models",
    icon: Search,
    accent: "orange",
    rotation: -0.25,
  },
  {
    title: "Native Desktop",
    description:
      "Tray integration, Preferences, cross-platform support, automatic updates, and native Electron performance.",
    tag: "Built Native",
    icon: MonitorCog,
    accent: "blue",
    rotation: 0.35,
  },
];

export function FeaturesSection() {
  return (
    <MotionConfig reducedMotion="user">
      <section
        id="features"
        aria-labelledby="features-title"
        className="landing-section-anchor relative overflow-hidden bg-cream pb-12 pt-2 sm:pb-16 sm:pt-4 lg:pb-20"
      >
        <SectionContainer className="relative">
          <div className="relative overflow-hidden rounded-[26px] border-[3px] border-ink bg-cream px-4 pb-6 pt-12 shadow-brutal-shell sm:px-6 sm:pb-8 sm:pt-14 lg:px-8 lg:pb-10 xl:px-10">
            <motion.header
              initial={{ opacity: 0, y: 22 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, amount: 0.45 }}
              transition={{ duration: 0.58, ease: [0.22, 1, 0.36, 1] }}
              className="relative z-10 mx-auto max-w-[850px] text-center"
            >
              <span className="inline-flex items-center gap-2 rounded-xl border-2 border-ink bg-yellow px-4 py-2 text-[0.7rem] font-black uppercase tracking-[-0.01em] shadow-brutal-sm sm:text-xs">
                What Ducky can do
              </span>
              <h2
                id="features-title"
                className="mt-5 text-[clamp(2.55rem,4.5vw,3.65rem)] font-black leading-[0.98] tracking-[-0.065em]"
              >
                <span className="block">Everything you need.</span>
                <span className="mt-1 block">
                  Right on your <span className="text-orange">desktop.</span>
                </span>
              </h2>
              <p className="mx-auto mt-5 max-w-[640px] text-sm font-semibold leading-[1.75] text-ink/78 sm:text-base">
                Ducky combines AI, productivity, desktop utilities, and smart
                reminders into one lightweight companion.
              </p>
            </motion.header>

            <div className="relative z-10 mt-10 grid min-w-0 gap-9 min-[1240px]:grid-cols-[minmax(0,2.2fr)_minmax(310px,0.8fr)] min-[1240px]:items-start min-[1240px]:gap-8">
              <div className="features-card-grid grid min-w-0 grid-cols-1 gap-5 md:grid-cols-2">
                {features.map((feature, index) => (
                  <FeatureCard key={feature.title} {...feature} index={index} />
                ))}
              </div>

              <FeaturesMascotPanel />
            </div>

            <CapabilityStrip />

            <motion.span
              aria-hidden="true"
              animate={{ rotate: [0, 15, 0] }}
              transition={{ duration: 7.5, repeat: Infinity, ease: "easeInOut" }}
              className="absolute left-[8%] top-[12%] hidden text-yellow drop-shadow-[2px_2px_0_#111] lg:block"
            >
              <Star className="size-7 fill-yellow" strokeWidth={2.2} />
            </motion.span>
            <span
              aria-hidden="true"
              className="absolute left-[18%] top-[25%] hidden size-4 rotate-12 rounded-[3px] border-2 border-ink bg-purple lg:block"
            />
            <Diamond
              aria-hidden="true"
              className="absolute right-[8%] top-[15%] hidden size-5 rotate-12 fill-pink text-ink lg:block"
              strokeWidth={2.2}
            />
            <Circle
              aria-hidden="true"
              className="absolute right-[34%] top-[21%] hidden size-3 fill-mint text-ink lg:block"
              strokeWidth={2.2}
            />
          </div>
        </SectionContainer>
      </section>
    </MotionConfig>
  );
}
