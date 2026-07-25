import { ArrowDownToLine, Check, type LucideIcon } from "lucide-react";
import { motion } from "framer-motion";

export type PlatformDownload = {
  title: string;
  subtitle: string;
  detail: string;
  buttonLabel: string;
  href: string;
  icon: LucideIcon;
  iconColor: string;
  chips: string[];
  rotation: number;
};

type PlatformCardProps = PlatformDownload & {
  index: number;
};

export function PlatformCard({
  title,
  subtitle,
  detail,
  buttonLabel,
  href,
  icon: Icon,
  iconColor,
  chips,
  rotation,
  index,
}: PlatformCardProps) {
  const hoverRotation = rotation < 0 ? -0.8 : 0.8;

  return (
    <motion.article
      initial={{ opacity: 0, y: 24, rotate: rotation }}
      whileInView={{ opacity: 1, y: 0, rotate: rotation }}
      viewport={{ once: true, amount: 0.2 }}
      whileHover={{ y: -7, rotate: hoverRotation }}
      transition={{
        duration: 0.5,
        delay: index * 0.07,
        ease: [0.22, 1, 0.36, 1],
      }}
      className="group flex min-h-[300px] min-w-0 flex-col rounded-[20px] border-[3px] border-ink bg-cream p-4 shadow-brutal-lg min-[360px]:p-5 sm:min-h-[330px] sm:p-6"
    >
      <div className="flex min-w-0 items-start gap-4">
        <span
          aria-hidden="true"
          className={`grid size-16 shrink-0 place-items-center rounded-[15px] border-[3px] border-ink shadow-brutal-sm transition-transform duration-300 group-hover:-rotate-2 min-[360px]:size-[72px] min-[360px]:rounded-[17px] ${iconColor}`}
        >
          <Icon className="size-8 min-[360px]:size-9" strokeWidth={2.35} />
        </span>

        <div className="min-w-0 pt-1">
          <h3 className="text-2xl font-black tracking-[-0.045em]">{title}</h3>
          <p className="mt-1.5 text-sm font-extrabold leading-snug">{subtitle}</p>
          <p className="mt-1 text-xs font-semibold leading-relaxed text-ink/68">
            {detail}
          </p>
        </div>
      </div>

      <motion.a
        href={href}
        aria-label={buttonLabel}
        whileHover={{ y: -3, scale: 1.012 }}
        whileTap={{ y: 1, scale: 0.99 }}
        transition={{ type: "spring", stiffness: 380, damping: 25 }}
        className="mt-7 flex min-h-14 items-center justify-center gap-3 rounded-[12px] border-2 border-ink bg-orange px-4 text-center text-sm font-black shadow-brutal outline-none focus-visible:ring-4 focus-visible:ring-orange/35"
      >
        {buttonLabel}
        <ArrowDownToLine aria-hidden="true" className="size-5" strokeWidth={2.7} />
      </motion.a>

      <div className="mt-auto flex flex-wrap gap-2.5 pt-5" aria-label="Compatibility">
        {chips.map((chip) => (
          <span
            key={chip}
            className="inline-flex min-h-8 items-center gap-1.5 rounded-lg border border-[#77b68f] bg-[#e6f5e9] px-2.5 py-1.5 text-[0.7rem] font-black text-[#175c38]"
          >
            <Check aria-hidden="true" className="size-3.5" strokeWidth={3} />
            {chip}
          </span>
        ))}
      </div>
    </motion.article>
  );
}
