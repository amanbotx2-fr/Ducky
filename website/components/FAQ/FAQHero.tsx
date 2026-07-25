"use client";

import Image from "next/image";
import {
  CircleHelp,
  Code2,
  LockKeyhole,
  ShieldCheck,
} from "lucide-react";
import { motion } from "framer-motion";
import { duckyFaq } from "../../lib/brandAssets";

const trustChips = [
  { label: "Free", icon: ShieldCheck },
  { label: "Open Source", icon: Code2 },
  { label: "Privacy First", icon: LockKeyhole },
] as const;

export function FAQHero() {
  return (
    <div className="relative grid min-w-0 gap-12 lg:grid-cols-[1.03fr_0.97fr] lg:items-center lg:gap-10 xl:gap-16">
      <motion.header
        initial={{ opacity: 0, x: -24 }}
        whileInView={{ opacity: 1, x: 0 }}
        viewport={{ once: true, amount: 0.3 }}
        transition={{ duration: 0.62, ease: [0.22, 1, 0.36, 1] }}
        className="relative z-10 min-w-0"
      >
        <span className="inline-flex items-center gap-2 rounded-xl border-2 border-ink bg-purple px-4 py-2.5 text-[0.7rem] font-black uppercase tracking-[-0.01em] shadow-brutal-sm sm:px-5 sm:text-xs">
          <CircleHelp aria-hidden="true" className="size-4" strokeWidth={2.8} />
          Got questions?
        </span>

        <h2
          id="faq-title"
          className="mt-7 max-w-[760px] text-[clamp(2.75rem,5.45vw,4.8rem)] font-black leading-[0.94] tracking-[-0.065em]"
        >
          <span className="block">Still curious?</span>
          <span className="mt-2 block text-orange">We&apos;ve got you.</span>
        </h2>

        <div className="mt-7 max-w-[620px] space-y-1 text-base font-semibold leading-[1.75] tracking-[-0.015em] text-ink/78 sm:text-lg">
          <p>Here are the most common questions about Ducky.</p>
          <p>If you need more help, we&apos;re always here for you.</p>
        </div>

        <ul
          aria-label="Ducky commitments"
          className="mt-7 flex flex-wrap gap-3"
        >
          {trustChips.map(({ label, icon: Icon }) => (
            <li
              key={label}
              className="inline-flex min-h-10 items-center gap-2 rounded-xl border border-[#77b68f] bg-[#e6f5e9] px-3.5 py-2 text-xs font-extrabold text-[#175c38] sm:text-sm"
            >
              <Icon aria-hidden="true" className="size-4" strokeWidth={2.6} />
              {label}
            </li>
          ))}
        </ul>
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
        aria-label="Ducky reading the frequently asked questions"
        className="relative mx-auto w-full max-w-[650px] pb-3 pt-20 sm:pt-16"
      >
        <motion.div
          animate={{ y: [0, -6, 0] }}
          transition={{ duration: 4.8, repeat: Infinity, ease: "easeInOut" }}
          className="pixel-text-bubble absolute right-0 top-0 z-20 flex h-[78px] w-[190px] items-center justify-center pb-2 shadow-brutal-sm sm:right-[2%] sm:w-[210px]"
        >
          <p className="text-sm font-black tracking-[-0.02em] sm:text-base">
            Ask away!
          </p>
        </motion.div>

        <div className="relative mx-auto max-w-[520px] overflow-hidden rounded-[26px] border-[3px] border-ink bg-cream shadow-brutal-window">
          <div className="flex h-[58px] items-center gap-3 border-b-[3px] border-ink bg-purple px-5">
            <span className="size-5 rounded-full border-2 border-ink bg-orange" />
            <span className="size-5 rounded-full border-2 border-ink bg-yellow" />
            <span className="size-5 rounded-full border-2 border-ink bg-mint" />
            <span className="ml-auto rounded-lg border-2 border-ink bg-cream/85 px-3 py-1 text-[0.62rem] font-black uppercase tracking-[0.12em]">
              Ducky.exe
            </span>
          </div>

          <div className="halftone relative grid min-h-[390px] place-items-center overflow-hidden bg-yellow/80 px-4 pb-4 pt-6 sm:min-h-[450px]">
            <div className="absolute inset-x-[13%] bottom-5 h-6 rounded-[50%] bg-orange/30" />
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
                src={duckyFaq}
                alt="Ducky, the official pixel-art mascot, reading an FAQ book"
                sizes="(max-width: 639px) 78vw, (max-width: 1023px) 460px, 34vw"
                className="h-auto max-h-[350px] w-[88%] object-contain drop-shadow-[0_12px_0_rgba(17,17,17,0.08)] sm:max-h-[405px]"
                unoptimized
              />
            </motion.div>
          </div>
        </div>

        <span
          aria-hidden="true"
          className="absolute left-[2%] top-[22%] hidden text-[3.5rem] font-black leading-none text-purple drop-shadow-[3px_3px_0_#111] sm:block"
        >
          ?
        </span>
      </motion.aside>
    </div>
  );
}
