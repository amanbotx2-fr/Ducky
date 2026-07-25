"use client";

import { Diamond, Sparkles, Star } from "lucide-react";
import { MotionConfig, motion } from "framer-motion";
import { SectionContainer } from "../SectionContainer";
import { FAQAccordion } from "./FAQAccordion";
import { FAQHero } from "./FAQHero";
import { FinalCTA } from "./FinalCTA";
import { HelpCards } from "./HelpCards";

export function FaqSection() {
  return (
    <MotionConfig reducedMotion="user">
      <section
        id="faq"
        aria-labelledby="faq-title"
        className="landing-section-anchor relative overflow-hidden bg-cream pb-14 pt-2 sm:pb-20 sm:pt-4"
      >
        <SectionContainer className="relative">
          <div className="relative overflow-hidden rounded-[26px] border-[3px] border-ink bg-cream px-4 pb-7 pt-12 shadow-brutal-shell sm:px-6 sm:pb-9 sm:pt-14 lg:px-8 lg:pb-11 xl:px-10">
            <FAQHero />
            <FAQAccordion />
            <HelpCards />
            <FinalCTA />

            <motion.span
              aria-hidden="true"
              animate={{ rotate: [0, 14, 0] }}
              transition={{
                duration: 7.4,
                repeat: Infinity,
                ease: "easeInOut",
              }}
              className="absolute left-[4%] top-[33%] hidden text-yellow drop-shadow-[2px_2px_0_#111] xl:block"
            >
              <Star className="size-7 fill-yellow" strokeWidth={2.2} />
            </motion.span>
            <Diamond
              aria-hidden="true"
              className="absolute right-[3%] top-[7%] hidden size-5 rotate-12 fill-purple text-ink lg:block"
              strokeWidth={2.2}
            />
            <Sparkles
              aria-hidden="true"
              className="absolute right-[4%] top-[31%] hidden size-7 text-orange xl:block"
              strokeWidth={2.3}
            />
          </div>
        </SectionContainer>
      </section>
    </MotionConfig>
  );
}
