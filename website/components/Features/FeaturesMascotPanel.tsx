import Image from "next/image";
import { Heart, Sparkles, Star } from "lucide-react";
import { motion } from "framer-motion";
import { mascot } from "../../lib/brandAssets";

export function FeaturesMascotPanel() {
  return (
    <motion.aside
      initial={{ opacity: 0, x: 24 }}
      whileInView={{ opacity: 1, x: 0 }}
      viewport={{ once: true, amount: 0.2 }}
      transition={{ duration: 0.65, delay: 0.18, ease: [0.22, 1, 0.36, 1] }}
      className="relative mx-auto w-full max-w-[430px] pt-20 min-[1240px]:max-w-none"
      aria-label="Ducky mascot showcase"
    >
      <motion.div
        animate={{ y: [0, -6, 0] }}
        transition={{ duration: 4.8, repeat: Infinity, ease: "easeInOut" }}
        className="pixel-text-bubble absolute left-1/2 top-0 z-20 flex h-[70px] w-[218px] -translate-x-1/2 items-center justify-center pb-2 shadow-brutal-sm"
      >
        <p className="text-sm font-black tracking-[-0.02em]">
          I&apos;m always here.
        </p>
      </motion.div>

      <div className="relative overflow-hidden rounded-[26px] border-[3px] border-ink bg-cream shadow-brutal-window">
        <div className="flex h-[56px] items-center gap-3 border-b-[3px] border-ink bg-purple px-5">
          <span className="size-5 rounded-full border-2 border-ink bg-orange" />
          <span className="size-5 rounded-full border-2 border-ink bg-yellow" />
          <span className="size-5 rounded-full border-2 border-ink bg-mint" />
          <span className="ml-auto rounded-lg border-2 border-ink bg-cream/85 px-3 py-1 text-[0.62rem] font-black uppercase tracking-[0.12em]">
            Ducky.exe
          </span>
        </div>

        <div className="halftone relative grid min-h-[430px] place-items-center overflow-hidden bg-yellow/80 px-5 pb-6 pt-8 sm:min-h-[500px] min-[1240px]:min-h-[550px]">
          <div className="absolute inset-x-[14%] bottom-7 h-6 rounded-[50%] bg-orange/35" />
          <motion.div
            animate={{ y: [0, -10, 0], rotate: [0, 0.5, 0] }}
            transition={{ duration: 4.6, repeat: Infinity, ease: "easeInOut" }}
            className="relative z-10 flex h-full w-full items-center justify-center"
          >
            <Image
              src={mascot}
              alt="Ducky, the official pixel-art desktop companion mascot"
              sizes="(max-width: 767px) 82vw, (max-width: 1239px) 420px, 29vw"
              className="h-auto max-h-[400px] w-[88%] object-contain drop-shadow-[0_14px_0_rgba(17,17,17,0.08)] min-[1240px]:max-h-[450px] min-[1240px]:w-[94%]"
              unoptimized
            />
          </motion.div>
        </div>
      </div>

      <motion.span
        aria-hidden="true"
        animate={{ rotate: [0, 14, 0] }}
        transition={{ duration: 7, repeat: Infinity, ease: "easeInOut" }}
        className="absolute -left-5 top-[31%] hidden text-yellow drop-shadow-[2px_2px_0_#111] sm:block"
      >
        <Star className="size-8 fill-yellow" strokeWidth={2.2} />
      </motion.span>
      <motion.span
        aria-hidden="true"
        animate={{ y: [0, -7, 0], rotate: [-8, 3, -8] }}
        transition={{ duration: 5.6, repeat: Infinity, ease: "easeInOut" }}
        className="absolute -bottom-5 right-1 text-pink drop-shadow-[2px_2px_0_#111]"
      >
        <Heart className="size-8 fill-pink" strokeWidth={2.2} />
      </motion.span>
      <span
        aria-hidden="true"
        className="absolute -right-3 top-[22%] size-4 rotate-45 rounded-[3px] border-2 border-ink bg-orange"
      />
      <Sparkles
        aria-hidden="true"
        className="absolute -right-5 bottom-[24%] hidden size-7 fill-orange/25 text-orange sm:block"
        strokeWidth={2.3}
      />
    </motion.aside>
  );
}
