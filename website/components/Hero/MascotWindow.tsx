import Image from "next/image";
import {
  Bell,
  Clock3,
  Heart,
  MessageSquareText,
  Sparkles,
  Star,
} from "lucide-react";
import { motion } from "framer-motion";
import { mascot } from "../../lib/brandAssets";
import { FloatingCard } from "./FloatingCard";

export function MascotWindow() {
  return (
    <div
      id="showcase"
      className="relative mx-auto w-full min-w-0 max-w-[760px]"
    >
      <motion.div
        initial={false}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        transition={{ duration: 0.68, delay: 0.16, ease: [0.22, 1, 0.36, 1] }}
        className="mascot-window-enter relative mr-0 overflow-hidden rounded-[28px] border-[3px] border-ink bg-cream shadow-brutal-window xl:mr-[25%]"
      >
        <div className="flex h-[56px] items-center gap-3 border-b-[3px] border-ink bg-purple px-5 sm:h-[62px]">
          <span className="size-5 rounded-full border-2 border-ink bg-orange" />
          <span className="size-5 rounded-full border-2 border-ink bg-yellow" />
          <span className="size-5 rounded-full border-2 border-ink bg-mint" />
          <span className="ml-auto rounded-lg border-2 border-ink bg-cream/85 px-3 py-1 text-[0.66rem] font-black uppercase tracking-[0.12em]">
            Ducky.exe
          </span>
        </div>

        <div className="halftone relative grid min-h-[440px] place-items-center overflow-hidden bg-yellow/80 px-5 pb-3 pt-8 sm:min-h-[500px] lg:min-h-[540px]">
          <div className="absolute inset-x-[13%] bottom-5 h-7 rounded-[50%] bg-orange/35 blur-[1px]" />
          <motion.div
            animate={{ y: [0, -10, 0], rotate: [0, 0.6, 0] }}
            transition={{
              duration: 4.6,
              repeat: Infinity,
              ease: "easeInOut",
            }}
            className="relative z-10 flex h-full w-full items-center justify-center"
          >
            <Image
              src={mascot}
              alt="Ducky, the official pixel-art desktop companion mascot"
              priority
              sizes="(max-width: 1024px) 74vw, 36vw"
              className="h-auto max-h-[430px] w-[92%] object-contain drop-shadow-[0_16px_0_rgba(17,17,17,0.08)] sm:w-[88%] xl:max-h-[470px]"
              unoptimized
            />
          </motion.div>
        </div>
      </motion.div>

      <motion.div
        initial={false}
        animate={{
          opacity: 1,
          scale: 1,
          x: 0,
          y: [0, -7, 0],
        }}
        transition={{
          opacity: { duration: 0.42, delay: 0.55 },
          scale: { duration: 0.42, delay: 0.55 },
          x: { duration: 0.42, delay: 0.55 },
          y: {
            duration: 4.2,
            delay: 1,
            repeat: Infinity,
            ease: "easeInOut",
          },
        }}
        className="pixel-bubble absolute left-[-2%] top-[18%] z-20 grid size-[90px] place-items-center bg-cream shadow-brutal sm:left-[-4%] sm:size-[106px] xl:left-[-8%]"
        aria-label="Ducky loves being your desktop buddy"
      >
        <Heart
          aria-hidden="true"
          className="size-10 fill-orange text-ink sm:size-12"
          strokeWidth={2.7}
        />
      </motion.div>

      <div className="relative z-20 mt-4 grid min-w-0 grid-cols-1 gap-3 sm:grid-cols-3 xl:absolute xl:right-0 xl:top-[7%] xl:mt-0 xl:w-[32%] xl:grid-cols-1 xl:gap-4">
        <FloatingCard
          title="AI Chat"
          description="Chat with multiple top AI models."
          icon={MessageSquareText}
          color="purple"
          delay={0.38}
          rotate={1.2}
        />
        <FloatingCard
          title="Pomodoro"
          description="Stay focused with smart sessions."
          icon={Clock3}
          color="mint"
          delay={0.48}
          rotate={2}
        />
        <FloatingCard
          title="Reminders"
          description="Never miss the things that matter."
          icon={Bell}
          color="yellow"
          delay={0.58}
          rotate={3.2}
        />
      </div>

      <motion.span
        aria-hidden="true"
        animate={{ rotate: [0, 14, 0] }}
        transition={{ duration: 7, repeat: Infinity, ease: "easeInOut" }}
        className="absolute -left-1 top-[4%] hidden text-yellow drop-shadow-[2px_2px_0_#111] sm:block"
      >
        <Star className="size-8 fill-yellow" strokeWidth={2.2} />
      </motion.span>
      <motion.span
        aria-hidden="true"
        animate={{ rotate: [0, -18, 0] }}
        transition={{ duration: 8, repeat: Infinity, ease: "easeInOut" }}
        className="absolute bottom-[7%] right-[5%] hidden text-orange xl:block"
      >
        <Sparkles className="size-7 fill-orange/25" strokeWidth={2.4} />
      </motion.span>
      <span
        aria-hidden="true"
        className="absolute right-[4%] top-[-2%] hidden size-4 rotate-45 rounded-[3px] border-2 border-ink bg-orange sm:block"
      />
    </div>
  );
}
