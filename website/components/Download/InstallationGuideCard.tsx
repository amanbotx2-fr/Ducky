import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";
import { motion } from "framer-motion";

export type InstallationStep = {
  title: string;
  description: ReactNode;
};

type InstallationGuideCardProps = {
  title: string;
  icon: LucideIcon;
  steps: InstallationStep[];
  note: ReactNode;
  noteColor: string;
  illustration: ReactNode;
  index: number;
};

export function InstallationGuideCard({
  title,
  icon: Icon,
  steps,
  note,
  noteColor,
  illustration,
  index,
}: InstallationGuideCardProps) {
  return (
    <motion.article
      initial={{ opacity: 0, y: 26 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, amount: 0.12 }}
      transition={{
        duration: 0.55,
        delay: index * 0.08,
        ease: [0.22, 1, 0.36, 1],
      }}
      className="min-w-0 rounded-[22px] border-[3px] border-ink bg-cream p-4 shadow-brutal-lg sm:p-6"
    >
      <div className="flex items-center gap-3">
        <span className="grid size-11 shrink-0 place-items-center rounded-[12px] border-2 border-ink bg-yellow shadow-brutal-sm">
          <Icon aria-hidden="true" className="size-6" strokeWidth={2.5} />
        </span>
        <h4 className="text-xl font-black tracking-[-0.035em] sm:text-2xl">
          {title}
        </h4>
      </div>

      <ol className="mt-6 grid min-w-0 grid-cols-1 overflow-hidden rounded-[15px] border-2 border-ink/45 sm:grid-cols-2">
        {steps.map((step, stepIndex) => (
          <li
            key={step.title}
            className={`relative min-h-[150px] p-4 ${
              stepIndex % 2 === 1 ? "sm:border-l sm:border-ink/35" : ""
            } ${stepIndex >= 2 ? "border-t border-ink/35" : stepIndex > 0 ? "border-t border-ink/35 sm:border-t-0" : ""}`}
          >
            <span className="absolute left-3.5 top-3.5 grid size-6 place-items-center rounded-full bg-ink text-[0.68rem] font-black text-cream">
              {stepIndex + 1}
            </span>
            <div className="pl-8">
              <h5 className="text-sm font-black">{step.title}</h5>
              <div className="mt-2 text-xs font-semibold leading-[1.65] text-ink/76">
                {step.description}
              </div>
            </div>
          </li>
        ))}
      </ol>

      <div className="mt-4">{illustration}</div>

      <div
        className={`mt-4 rounded-[13px] border-2 border-ink/55 p-3.5 text-xs font-bold leading-relaxed ${noteColor}`}
      >
        {note}
      </div>
    </motion.article>
  );
}
