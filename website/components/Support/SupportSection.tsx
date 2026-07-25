"use client";

import { MotionConfig } from "framer-motion";
import { SectionContainer } from "../SectionContainer";
import { SupportBenefits } from "./SupportBenefits";
import { SupportCards } from "./SupportCards";
import { SupportHero } from "./SupportHero";

export function SupportSection() {
  return (
    <MotionConfig reducedMotion="user">
      <section
        id="support"
        aria-labelledby="support-ducky-title"
        className="landing-section-anchor relative overflow-hidden bg-cream pb-14 pt-2 sm:pb-20 sm:pt-4"
      >
        <SectionContainer className="relative">
          <div className="relative overflow-hidden rounded-[26px] border-[3px] border-ink bg-cream px-4 pb-7 pt-12 shadow-brutal-shell sm:px-6 sm:pb-9 sm:pt-14 lg:px-8 lg:pb-11 xl:px-10">
            <SupportHero />
            <SupportCards />
            <SupportBenefits />
          </div>
        </SectionContainer>
      </section>
    </MotionConfig>
  );
}
