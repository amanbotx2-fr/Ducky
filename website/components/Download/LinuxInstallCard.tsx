import { Check, TerminalSquare } from "lucide-react";
import { motion } from "framer-motion";
import { releaseAssets } from "../../lib/releaseAssets";

export function LinuxInstallCard() {
  const appImageName = `Ducky-${releaseAssets.version}-x86_64.AppImage`;

  return (
    <motion.article
      initial={{ opacity: 0, y: 24 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, amount: 0.2 }}
      transition={{ duration: 0.55, ease: [0.22, 1, 0.36, 1] }}
      className="mt-6 grid min-w-0 gap-6 rounded-[20px] border-[3px] border-orange bg-cream p-5 shadow-brutal-lg md:grid-cols-[1fr_1.1fr_0.9fr] md:items-center sm:p-6"
    >
      <div className="flex min-w-0 items-start gap-3">
        <span className="grid size-12 shrink-0 place-items-center rounded-[13px] border-2 border-ink bg-yellow shadow-brutal-sm">
          <TerminalSquare aria-hidden="true" className="size-7" strokeWidth={2.4} />
        </span>
        <div>
          <h4 className="text-xl font-black tracking-[-0.035em]">
            Linux — AppImage
          </h4>
          <p className="mt-2 text-xs font-semibold leading-relaxed text-ink/72">
            Download the portable AppImage, make it executable, then run it.
          </p>
        </div>
      </div>

      <pre className="min-w-0 overflow-hidden rounded-[12px] border-2 border-ink bg-ink px-4 py-3 text-[0.72rem] font-bold leading-[1.8] text-cream shadow-brutal-sm sm:overflow-x-auto sm:text-xs">
        <code className="block whitespace-pre-wrap break-all sm:whitespace-pre sm:break-normal">{`chmod +x ${appImageName}
./${appImageName}`}</code>
      </pre>

      <div className="min-w-0">
        <p className="flex items-start gap-2 text-sm font-black">
          <Check
            aria-hidden="true"
            className="mt-0.5 size-5 shrink-0 rounded-full bg-mint p-0.5"
            strokeWidth={3}
          />
          No installation required.
        </p>
        <p className="mt-2 text-xs font-semibold leading-relaxed text-ink/72">
          Runs on most modern Linux distributions.
        </p>
        <a
          href={releaseAssets.linux}
          className="mt-2 inline-flex min-h-11 items-center rounded-lg text-xs font-black underline decoration-2 underline-offset-4 outline-none focus-visible:ring-4 focus-visible:ring-orange/35"
        >
          Download the AppImage
        </a>
      </div>
    </motion.article>
  );
}
