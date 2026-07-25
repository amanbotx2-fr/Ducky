"use client";

import {
  Code2,
  MonitorSmartphone,
  RefreshCw,
  ShieldCheck,
  Zap,
} from "lucide-react";
import { motion, MotionConfig } from "framer-motion";
import { CapabilityStrip, type Capability } from "../Features/CapabilityStrip";
import { SectionContainer } from "../SectionContainer";
import { DownloadHeader } from "./DownloadHeader";
import { InstallationHelp } from "./InstallationHelp";
import { PlatformDownloads } from "./PlatformDownloads";
import { SupportSection } from "./SupportSection";

const downloadBenefits: Capability[] = [
  {
    title: "Lightning Fast",
    description: "A quick download with no account setup.",
    icon: Zap,
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
    description: "Ducky runs locally on your desktop.",
    icon: ShieldCheck,
    color: "bg-pink",
  },
  {
    title: "Cross Platform",
    description: "Available for macOS, Windows, and Linux.",
    icon: MonitorSmartphone,
    color: "bg-mint",
  },
  {
    title: "Always Improving",
    description: "Regular releases and secure update foundations.",
    icon: RefreshCw,
    color: "bg-blue",
  },
];

export function DownloadSection() {
  return (
    <MotionConfig reducedMotion="user">
      <section
        id="download"
        aria-labelledby="download-title"
        className="landing-section-anchor relative overflow-hidden bg-cream pb-14 pt-2 sm:pb-20 sm:pt-4"
      >
        <SectionContainer className="relative">
          <div className="relative overflow-hidden rounded-[26px] border-[3px] border-ink bg-cream px-4 pb-7 pt-12 shadow-brutal-shell sm:px-6 sm:pb-9 sm:pt-14 lg:px-8 lg:pb-11 xl:px-10">
            <DownloadHeader />
            <PlatformDownloads />
            <InstallationHelp />
            <SupportSection />
            <CapabilityStrip
              heading="Download with confidence."
              items={downloadBenefits}
              gridClassName="xl:grid-cols-5 xl:gap-0"
              className="mt-8"
            />

            <motion.span
              aria-hidden="true"
              animate={{ rotate: [0, 14, 0] }}
              transition={{ duration: 7.5, repeat: Infinity, ease: "easeInOut" }}
              className="absolute bottom-[11%] right-[3%] hidden size-4 rotate-45 rounded-[3px] border-2 border-ink bg-purple lg:block"
            />
          </div>
        </SectionContainer>
      </section>
    </MotionConfig>
  );
}
