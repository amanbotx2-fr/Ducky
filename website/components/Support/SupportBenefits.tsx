"use client";

import {
  Code2,
  HeartHandshake,
  Rocket,
  ShieldCheck,
  Star,
} from "lucide-react";
import {
  CapabilityStrip,
  type Capability,
} from "../Features/CapabilityStrip";

const supportBenefits: Capability[] = [
  {
    title: "Build More Features",
    description: "New ideas, tools, and smart improvements.",
    icon: Code2,
    color: "bg-purple",
  },
  {
    title: "Stay Independent",
    description: "No ads. No tracking. Just a happy duck.",
    icon: ShieldCheck,
    color: "bg-mint",
  },
  {
    title: "Faster Updates",
    description: "Fixes, updates, and new features sooner.",
    icon: Rocket,
    color: "bg-orange",
  },
  {
    title: "Community First",
    description: "You are part of the Ducky family.",
    icon: HeartHandshake,
    color: "bg-pink",
  },
  {
    title: "Open Source Forever",
    description: "Free, transparent, and built for everyone.",
    icon: Star,
    color: "bg-yellow",
  },
];

export function SupportBenefits() {
  return (
    <CapabilityStrip
      heading="Your support helps Ducky stay independent and awesome."
      items={supportBenefits}
      gridClassName="xl:grid-cols-5 xl:gap-0"
      className="mt-8"
    />
  );
}
