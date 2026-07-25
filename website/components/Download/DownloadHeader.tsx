import { Check, Diamond, Heart, Square, Star } from "lucide-react";
import { motion } from "framer-motion";
import { Badge } from "../Badge";
import { DownloadMascotWindow } from "./DownloadMascotWindow";

const trustPoints = ["Free", "Open Source", "No Sign Up"];

export function DownloadHeader() {
  return (
    <div className="relative grid min-w-0 gap-10 lg:grid-cols-[1.15fr_0.85fr] lg:items-center lg:gap-12">
      <motion.header
        initial={{ opacity: 0, x: -24 }}
        whileInView={{ opacity: 1, x: 0 }}
        viewport={{ once: true, amount: 0.35 }}
        transition={{ duration: 0.62, ease: [0.22, 1, 0.36, 1] }}
        className="relative z-10 min-w-0"
      >
        <Badge compact>Download Ducky</Badge>

        <h2
          id="download-title"
          className="mt-7 max-w-[780px] text-[clamp(2.65rem,5.4vw,4.6rem)] font-black leading-[0.94] tracking-[-0.065em]"
        >
          <span className="block">Download Ducky.</span>
          <span className="mt-2 block">
            Bring your <span className="text-orange">buddy</span> home.
          </span>
        </h2>

        <p className="mt-7 max-w-[620px] text-base font-semibold leading-[1.75] tracking-[-0.015em] text-ink/78 sm:text-lg">
          Ducky runs locally on your desktop. Choose your platform below and
          get started in less than a minute.
        </p>

        <div className="mt-6 inline-flex flex-wrap items-center gap-x-3 gap-y-2 rounded-xl border border-[#77b68f] bg-[#e6f5e9] px-3.5 py-2.5 text-xs font-extrabold text-[#175c38] sm:text-sm">
          {trustPoints.map((point, index) => (
            <span key={point} className="inline-flex items-center gap-1.5">
              {index === 0 && (
                <Check aria-hidden="true" className="size-4" strokeWidth={3} />
              )}
              {point}
              {index !== trustPoints.length - 1 && (
                <span aria-hidden="true" className="ml-1 text-[#4a8d64]">
                  •
                </span>
              )}
            </span>
          ))}
        </div>
      </motion.header>

      <DownloadMascotWindow />

      <motion.span
        aria-hidden="true"
        animate={{ rotate: [0, 16, 0] }}
        transition={{ duration: 8, repeat: Infinity, ease: "easeInOut" }}
        className="absolute left-[45%] top-[10%] hidden text-yellow drop-shadow-[2px_2px_0_#111] xl:block"
      >
        <Star className="size-7 fill-yellow" strokeWidth={2.2} />
      </motion.span>
      <span
        aria-hidden="true"
        className="absolute left-[47%] top-[50%] hidden size-4 rotate-12 rounded-[3px] border-2 border-ink bg-mint lg:block"
      />
      <Diamond
        aria-hidden="true"
        className="absolute right-[2%] top-[12%] hidden size-5 rotate-12 fill-pink text-ink lg:block"
        strokeWidth={2.2}
      />
      <Square
        aria-hidden="true"
        className="absolute right-[46%] top-[25%] hidden size-4 rotate-12 fill-purple text-ink xl:block"
        strokeWidth={2.2}
      />
      <Heart
        aria-hidden="true"
        className="absolute right-[1%] bottom-[4%] hidden size-6 fill-pink text-ink lg:block"
        strokeWidth={2.2}
      />
    </div>
  );
}
