import Image from "next/image";
import { mascot } from "../lib/brandAssets";

type BrandMarkProps = {
  compact?: boolean;
};

export function BrandMark({ compact = false }: BrandMarkProps) {
  return (
    <a
      href="#top"
      className="group inline-flex min-w-0 items-center gap-3 rounded-xl focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-orange/30"
      aria-label="Ducky home"
    >
      <span
        className={`relative grid shrink-0 place-items-center overflow-hidden rounded-[14px] border-2 border-ink bg-cream shadow-brutal-sm transition-transform group-hover:-translate-y-0.5 ${
          compact ? "size-12" : "size-12 sm:size-[62px]"
        }`}
      >
        <Image
          src={mascot}
          alt=""
          sizes={compact ? "48px" : "62px"}
          className="h-[88%] w-[88%] object-contain"
          priority
          unoptimized
        />
      </span>
      <span className="min-w-0 leading-none">
        <span
          className={`block font-black tracking-[-0.055em] ${
            compact ? "text-[1.6rem]" : "text-[1.75rem] sm:text-[2.25rem]"
          }`}
        >
          Ducky
        </span>
        <span
          className={`mt-1.5 block truncate font-extrabold uppercase leading-tight tracking-[-0.025em] text-orange ${
            compact
              ? "text-[0.58rem]"
              : "hidden text-[0.69rem] min-[460px]:block"
          }`}
        >
          AI Desktop Companion
        </span>
      </span>
    </a>
  );
}
