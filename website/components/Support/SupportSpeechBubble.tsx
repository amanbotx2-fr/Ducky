"use client";

import { motion } from "framer-motion";

export function SupportSpeechBubble() {
  return (
    <motion.div
      animate={{ y: [0, -6, 0] }}
      transition={{ duration: 4.8, repeat: Infinity, ease: "easeInOut" }}
      className="pixel-text-bubble relative z-20 flex min-h-[112px] w-[230px] items-center justify-center px-8 pb-3 pt-2 shadow-brutal-sm sm:w-[250px]"
    >
      <p className="text-sm font-black leading-[1.55] tracking-[-0.02em] sm:text-base">
        <span className="block">Thanks a latte!</span>
        <span className="block">You make Ducky</span>
        <span className="block">
          quack-tastic! <span aria-hidden="true">💛</span>
        </span>
      </p>
    </motion.div>
  );
}
