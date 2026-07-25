"use client";

import {
  Code2,
  Heart,
  LockKeyhole,
  Monitor,
  ShieldAlert,
  ShieldCheck,
  Sparkles,
  Users,
  type LucideIcon,
} from "lucide-react";
import type { ReactNode } from "react";
import { FAQItem } from "./FAQItem";

type FAQData = {
  question: string;
  answer: ReactNode;
  icon: LucideIcon;
  color: string;
};

const questions: FAQData[] = [
  {
    question: "Is Ducky really free?",
    answer: (
      <>
        Yes. Ducky is completely free and open source. No subscriptions, no
        hidden fees, and no premium tiers.
      </>
    ),
    icon: Heart,
    color: "bg-purple",
  },
  {
    question: "Does Ducky collect my data?",
    answer: (
      <>
        No. Ducky doesn&apos;t track you or collect analytics. Your conversations
        only go to whichever AI provider you configure.
      </>
    ),
    icon: LockKeyhole,
    color: "bg-yellow",
  },
  {
    question: "Which AI providers are supported?",
    answer: (
      <>
        OpenAI, Gemini, Grok, OpenRouter, Ollama, LM Studio, and
        OpenAI-compatible endpoints.
      </>
    ),
    icon: Sparkles,
    color: "bg-mint",
  },
  {
    question: "Which operating systems work?",
    answer: <>Ducky works on macOS, Windows, and Linux.</>,
    icon: Monitor,
    color: "bg-pink",
  },
  {
    question: "Why does macOS say Ducky can’t be opened?",
    answer: (
      <>
        Ducky isn&apos;t code signed yet. Open it once from System Settings,
        Privacy &amp; Security, then choose Open Anyway. After that it launches
        normally.
      </>
    ),
    icon: ShieldAlert,
    color: "bg-orange",
  },
  {
    question: "Why does Windows show SmartScreen?",
    answer: (
      <>
        Windows displays SmartScreen for many newly released apps without an
        established reputation. Choose More info, then Run anyway.
      </>
    ),
    icon: ShieldCheck,
    color: "bg-mint",
  },
  {
    question: "Is Ducky open source?",
    answer: (
      <>
        Yes. Everything is available on GitHub. Anyone can inspect, contribute,
        or improve it.
      </>
    ),
    icon: Code2,
    color: "bg-purple",
  },
  {
    question: "Can I contribute?",
    answer: (
      <>
        Absolutely. Bug reports, feature requests, pull requests,
        documentation, and community feedback are all welcome.
      </>
    ),
    icon: Users,
    color: "bg-orange",
  },
];

export function FAQAccordion() {
  return (
    <section aria-label="Frequently asked questions" className="mt-12">
      <div className="grid grid-cols-1 items-start gap-4 md:grid-cols-2 lg:gap-5">
        {questions.map((item) => (
          <FAQItem key={item.question} {...item} />
        ))}
      </div>
    </section>
  );
}
