import { AlertTriangle, Check, ShieldCheck } from "lucide-react";

export function MacSecurityIllustration() {
  return (
    <div
      aria-hidden="true"
      className="grid min-w-0 gap-3 rounded-[14px] border-2 border-ink/70 bg-purple/12 p-3 sm:grid-cols-[1.05fr_0.95fr]"
    >
      <div className="rounded-[11px] border-2 border-ink bg-[#343434] p-3 text-cream shadow-brutal-sm">
        <div className="flex items-center gap-2">
          <AlertTriangle className="size-5 shrink-0 text-yellow" strokeWidth={2.4} />
          <span className="text-[0.67rem] font-black">Ducky can&apos;t be opened</span>
        </div>
        <p className="mt-2 text-[0.57rem] font-semibold leading-relaxed text-cream/78">
          Apple cannot verify this app for malicious software.
        </p>
        <span className="mt-3 block rounded-md bg-blue px-3 py-1.5 text-center text-[0.58rem] font-black text-ink">
          OK
        </span>
      </div>

      <div className="rounded-[11px] border-2 border-ink bg-cream p-3">
        <p className="text-[0.58rem] font-black uppercase tracking-[0.08em] text-ink/62">
          Privacy &amp; Security
        </p>
        <div className="mt-3 flex items-center gap-2">
          <ShieldCheck className="size-5 shrink-0" strokeWidth={2.4} />
          <p className="text-[0.58rem] font-semibold leading-relaxed">
            Ducky was blocked to protect your Mac.
          </p>
        </div>
        <span className="mt-3 block rounded-md border-2 border-ink bg-ink px-2 py-1.5 text-center text-[0.58rem] font-black text-cream">
          Open Anyway
        </span>
      </div>
    </div>
  );
}

export function WindowsSecurityIllustration() {
  return (
    <div
      aria-hidden="true"
      className="grid min-w-0 gap-3 rounded-[14px] border-2 border-[#77b68f] bg-[#e6f5e9] p-3 sm:grid-cols-[1.05fr_0.95fr]"
    >
      <div className="rounded-[11px] border-2 border-ink bg-[#0d477f] p-3 text-cream shadow-brutal-sm">
        <div className="flex items-center gap-2">
          <ShieldCheck className="size-5 shrink-0 text-blue" strokeWidth={2.4} />
          <span className="text-[0.67rem] font-black">
            Windows protected your PC
          </span>
        </div>
        <p className="mt-2 text-[0.57rem] font-semibold leading-relaxed text-cream/82">
          Microsoft Defender SmartScreen prevented an unrecognized app from
          starting.
        </p>
        <span className="mt-3 block text-[0.58rem] font-black underline underline-offset-2">
          More info
        </span>
      </div>

      <div className="flex flex-col justify-between rounded-[11px] border-2 border-ink bg-cream p-3">
        <div className="flex items-center gap-2">
          <Check
            className="size-5 shrink-0 rounded-md border border-ink bg-mint p-0.5"
            strokeWidth={3}
          />
          <p className="text-[0.58rem] font-semibold leading-relaxed">
            Publisher: Unknown publisher
          </p>
        </div>
        <span className="mt-3 block rounded-md border-2 border-ink bg-blue px-2 py-1.5 text-center text-[0.58rem] font-black">
          Run anyway
        </span>
      </div>
    </div>
  );
}
