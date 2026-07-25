import Image from "next/image";
import { motion } from "framer-motion";
import { mascot } from "../../lib/brandAssets";

export function DownloadMascotWindow() {
  return (
    <motion.aside
      initial={{ opacity: 0, x: 24 }}
      whileInView={{ opacity: 1, x: 0 }}
      viewport={{ once: true, amount: 0.25 }}
      transition={{ duration: 0.65, delay: 0.15, ease: [0.22, 1, 0.36, 1] }}
      className="relative mx-auto w-full max-w-[450px] pt-20"
      aria-label="Ducky waiting beside the download guide"
    >
      <motion.div
        animate={{ y: [0, -6, 0] }}
        transition={{ duration: 4.5, repeat: Infinity, ease: "easeInOut" }}
        className="pixel-text-bubble absolute left-1/2 top-0 z-20 flex h-[70px] w-[210px] -translate-x-1/2 items-center justify-center pb-2 shadow-brutal-sm"
      >
        <p className="text-sm font-black tracking-[-0.02em]">
          I&apos;ll be waiting.
        </p>
      </motion.div>

      <div className="relative overflow-hidden rounded-[26px] border-[3px] border-ink bg-cream shadow-brutal-window">
        <div className="flex h-[58px] items-center gap-3 border-b-[3px] border-ink bg-purple px-5">
          <span className="size-5 rounded-full border-2 border-ink bg-orange" />
          <span className="size-5 rounded-full border-2 border-ink bg-yellow" />
          <span className="size-5 rounded-full border-2 border-ink bg-mint" />
          <span className="ml-auto rounded-lg border-2 border-ink bg-cream/85 px-3 py-1 text-[0.62rem] font-black uppercase tracking-[0.12em]">
            Ducky.exe
          </span>
        </div>

        <div className="halftone relative grid min-h-[340px] place-items-center overflow-hidden bg-yellow/80 px-5 pb-5 pt-7 min-[380px]:min-h-[370px] sm:min-h-[430px]">
          <div className="absolute inset-x-[14%] bottom-6 h-6 rounded-[50%] bg-orange/35" />
          <motion.div
            animate={{ y: [0, -9, 0], rotate: [0, 0.5, 0] }}
            transition={{ duration: 4.6, repeat: Infinity, ease: "easeInOut" }}
            className="relative z-10 flex h-full w-full items-center justify-center"
          >
            <Image
              src={mascot}
              alt="Ducky, the official pixel-art desktop companion mascot"
              sizes="(max-width: 767px) 82vw, 430px"
              className="h-auto w-auto max-h-[360px] max-w-[92%] object-contain drop-shadow-[0_14px_0_rgba(17,17,17,0.08)] sm:max-h-[390px]"
              unoptimized
            />
          </motion.div>
        </div>
      </div>
    </motion.aside>
  );
}
