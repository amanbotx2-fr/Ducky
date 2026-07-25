"use client";

import { motion, MotionConfig } from "framer-motion";
import { Check, Diamond, Square, Star } from "lucide-react";
import { Badge } from "../Badge";
import { Navbar } from "../Navbar/Navbar";
import { SectionContainer } from "../SectionContainer";
import { FeatureStrip } from "./FeatureStrip";
import { HeroButtons } from "./HeroButtons";
import { MascotWindow } from "./MascotWindow";

const trustPoints = ["Free", "Open Source", "No Sign Up"];

export function Hero() {
  return (
    <MotionConfig reducedMotion="user">
      <section
        id="top"
        aria-labelledby="hero-title"
        className="relative min-h-screen overflow-hidden bg-cream py-3 sm:py-5 lg:py-7"
      >
        <div
          aria-hidden="true"
          className="page-confetti pointer-events-none absolute inset-0 opacity-70"
        />

        <SectionContainer className="relative">
          <div className="relative rounded-[26px] border-[3px] border-ink bg-cream px-4 pb-5 pt-4 shadow-brutal-shell sm:px-6 sm:pb-7 sm:pt-5 lg:px-8 lg:pb-0 xl:px-10">
            <Navbar />

            <div className="relative z-10 grid min-w-0 gap-10 pb-2 pt-10 xl:grid-cols-[0.9fr_1.1fr] xl:items-center xl:gap-10 xl:pb-0 xl:pt-12">
              <motion.div
                initial={false}
                animate={{ opacity: 1, x: 0 }}
                transition={{
                  duration: 0.65,
                  delay: 0.12,
                  ease: [0.22, 1, 0.36, 1],
                }}
                className="hero-copy-enter relative z-20 min-w-0 xl:pb-12"
              >
                <Badge>Your desktop. Your buddy.</Badge>

                <h1
                  id="hero-title"
                  className="mt-7 max-w-[700px] text-[clamp(3.15rem,7.1vw,5.8rem)] font-black leading-[0.9] tracking-[-0.07em] xl:text-[clamp(4rem,5.2vw,5.3rem)]"
                >
                  <span className="block">Your Desktop.</span>
                  <span className="mt-2 block">Your Buddy.</span>
                  <span className="mt-3 block text-orange">
                    That&apos;s Ducky.
                  </span>
                </h1>

                <p className="mt-7 max-w-[540px] text-base font-semibold leading-[1.75] tracking-[-0.015em] text-ink/78 sm:text-lg">
                  An AI companion that lives on your desktop, chats with you,
                  keeps you focused, and makes work a little more fun.
                </p>

                <div className="mt-8">
                  <HeroButtons />
                </div>

                <div className="mt-6 inline-flex flex-wrap items-center gap-x-3 gap-y-2 rounded-xl border border-[#77b68f] bg-[#e6f5e9] px-3.5 py-2.5 text-xs font-extrabold text-[#175c38] sm:text-sm">
                  {trustPoints.map((point, index) => (
                    <span
                      key={point}
                      className="inline-flex items-center gap-1.5"
                    >
                      {index === 0 && (
                        <Check
                          aria-hidden="true"
                          className="size-4"
                          strokeWidth={3}
                        />
                      )}
                      {point}
                      {index !== trustPoints.length - 1 && (
                        <span
                          aria-hidden="true"
                          className="ml-1 text-[#4a8d64]"
                        >
                          •
                        </span>
                      )}
                    </span>
                  ))}
                </div>
              </motion.div>

              <MascotWindow />
            </div>

            <FeatureStrip />

            <motion.span
              aria-hidden="true"
              animate={{ rotate: [0, 16, 0] }}
              transition={{ duration: 8, repeat: Infinity, ease: "easeInOut" }}
              className="absolute left-[40%] top-[22%] hidden text-yellow drop-shadow-[2px_2px_0_#111] xl:block"
            >
              <Star className="size-7 fill-yellow" strokeWidth={2.2} />
            </motion.span>
            <motion.span
              aria-hidden="true"
              animate={{ y: [0, -8, 0] }}
              transition={{
                duration: 5.5,
                repeat: Infinity,
                ease: "easeInOut",
              }}
              className="absolute left-[42%] top-[48%] hidden rotate-12 text-purple lg:block"
            >
              <Square className="size-6 fill-purple" strokeWidth={2.4} />
            </motion.span>
            <motion.span
              aria-hidden="true"
              animate={{ rotate: [45, 60, 45] }}
              transition={{ duration: 7, repeat: Infinity, ease: "easeInOut" }}
              className="absolute right-[3.5%] top-[17%] hidden rotate-45 text-orange xl:block"
            >
              <Diamond
                className="size-5 fill-orange/65"
                strokeWidth={2.4}
              />
            </motion.span>
          </div>
        </SectionContainer>
      </section>
    </MotionConfig>
  );
}
