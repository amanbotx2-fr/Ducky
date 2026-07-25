"use client";

import Image from "next/image";
import { Circle, Diamond, Heart, Sparkles, Star } from "lucide-react";
import { motion } from "framer-motion";
import { duckyCoffee } from "../../lib/brandAssets";
import { SupportSpeechBubble } from "./SupportSpeechBubble";

export function SupportHero() {
  return (
    <div className="relative grid min-w-0 gap-12 lg:grid-cols-[1.02fr_0.98fr] lg:items-start lg:gap-10 xl:gap-16">
      <motion.header
        initial={{ opacity: 0, x: -24 }}
        whileInView={{ opacity: 1, x: 0 }}
        viewport={{ once: true, amount: 0.3 }}
        transition={{ duration: 0.62, ease: [0.22, 1, 0.36, 1] }}
        className="relative z-10 min-w-0 lg:pt-5"
      >
        <span className="inline-flex items-center gap-2 rounded-xl border-2 border-ink bg-yellow px-4 py-2.5 text-[0.7rem] font-black uppercase tracking-[-0.01em] shadow-brutal-sm sm:px-5 sm:text-xs">
          <Heart
            aria-hidden="true"
            className="size-4 fill-pink"
            strokeWidth={2.6}
          />
          Love Ducky? Support its journey.
        </span>

        <h2
          id="support-ducky-title"
          className="mt-7 max-w-[760px] text-[clamp(2.65rem,5.35vw,4.6rem)] font-black leading-[0.95] tracking-[-0.065em]"
        >
          <span className="block">Buy Ducky a Coffee.</span>
          <span className="mt-2 block">
            Fuel <span className="text-orange">more features.</span>
          </span>
        </h2>

        <div className="mt-7 max-w-[640px] space-y-1 text-base font-semibold leading-[1.75] tracking-[-0.015em] text-ink/78 sm:text-lg">
          <p>Ducky is free and open source.</p>
          <p>
            Your support helps keep it independent, ad-free, and full of
            delightful features.
          </p>
          <p>
            Every coffee helps this little duck build bigger things.{" "}
            <span aria-hidden="true">🐣</span>
          </p>
        </div>
      </motion.header>

      <motion.aside
        initial={{ opacity: 0, x: 24 }}
        whileInView={{ opacity: 1, x: 0 }}
        viewport={{ once: true, amount: 0.2 }}
        transition={{
          duration: 0.66,
          delay: 0.14,
          ease: [0.22, 1, 0.36, 1],
        }}
        aria-label="Ducky enjoying a coffee"
        className="relative mx-auto w-full max-w-[690px] pb-3 pt-3"
      >
        <div className="grid items-center gap-5 sm:grid-cols-[minmax(0,1fr)_230px] lg:grid-cols-1 xl:grid-cols-[minmax(0,1fr)_235px]">
          <div className="relative overflow-hidden rounded-[26px] border-[3px] border-ink bg-cream shadow-brutal-window">
            <div className="flex h-[58px] items-center gap-3 border-b-[3px] border-ink bg-purple px-5">
              <span className="size-5 rounded-full border-2 border-ink bg-orange" />
              <span className="size-5 rounded-full border-2 border-ink bg-yellow" />
              <span className="size-5 rounded-full border-2 border-ink bg-mint" />
              <span className="ml-auto rounded-lg border-2 border-ink bg-cream/85 px-3 py-1 text-[0.62rem] font-black uppercase tracking-[0.12em]">
                Ducky.exe
              </span>
            </div>

            <div className="halftone relative grid min-h-[350px] place-items-center overflow-hidden bg-yellow/80 px-4 pb-4 pt-5 sm:min-h-[390px] lg:min-h-[430px]">
              <div className="absolute inset-x-[15%] bottom-5 h-6 rounded-[50%] bg-orange/30" />
              <motion.div
                animate={{ y: [0, -9, 0], rotate: [0, 0.45, 0] }}
                transition={{
                  duration: 4.7,
                  repeat: Infinity,
                  ease: "easeInOut",
                }}
                className="relative z-10 flex h-full w-full items-center justify-center"
              >
                <Image
                  src={duckyCoffee}
                  alt="Ducky, the official pixel-art mascot, happily holding a coffee"
                  sizes="(max-width: 639px) 82vw, (max-width: 1023px) 48vw, (max-width: 1279px) 38vw, 32vw"
                  className="h-auto max-h-[330px] w-[94%] object-contain drop-shadow-[0_12px_0_rgba(17,17,17,0.08)] sm:max-h-[365px] lg:max-h-[400px]"
                  unoptimized
                />
              </motion.div>
            </div>
          </div>

          <div className="mx-auto sm:-ml-8 lg:mx-auto lg:-mt-6 xl:ml-0 xl:mt-0">
            <SupportSpeechBubble />
          </div>
        </div>
      </motion.aside>

      <motion.span
        aria-hidden="true"
        animate={{ rotate: [0, 15, 0] }}
        transition={{ duration: 7.3, repeat: Infinity, ease: "easeInOut" }}
        className="absolute left-[46%] top-[15%] hidden text-yellow drop-shadow-[2px_2px_0_#111] xl:block"
      >
        <Star className="size-7 fill-yellow" strokeWidth={2.2} />
      </motion.span>
      <Diamond
        aria-hidden="true"
        className="absolute right-[47%] top-[4%] hidden size-5 rotate-12 fill-pink text-ink xl:block"
        strokeWidth={2.2}
      />
      <Circle
        aria-hidden="true"
        className="absolute right-[1%] top-[9%] hidden size-5 fill-mint text-ink lg:block"
        strokeWidth={2.2}
      />
      <span
        aria-hidden="true"
        className="absolute right-[4%] bottom-[17%] hidden size-4 rounded-full border-2 border-ink bg-orange lg:block"
      />
      <Sparkles
        aria-hidden="true"
        className="absolute bottom-[9%] left-[48%] hidden size-7 text-orange xl:block"
        strokeWidth={2.3}
      />
    </div>
  );
}
