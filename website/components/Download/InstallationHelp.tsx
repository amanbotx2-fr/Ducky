import { CircleHelp, Lightbulb, Monitor } from "lucide-react";
import { motion } from "framer-motion";
import { AppleLogoIcon } from "../icons/AppleLogoIcon";
import {
  MacSecurityIllustration,
  WindowsSecurityIllustration,
} from "./SecurityDialogIllustrations";
import {
  InstallationGuideCard,
  type InstallationStep,
} from "./InstallationGuideCard";
import { LinuxInstallCard } from "./LinuxInstallCard";

const macSteps: InstallationStep[] = [
  {
    title: "Download Ducky",
    description: "Open the downloaded Ducky .dmg file.",
  },
  {
    title: "Move to Applications",
    description: "Drag Ducky into your Applications folder.",
  },
  {
    title: "Open once",
    description: (
      <>
        Open Ducky. macOS may say,{" "}
        <q>Ducky can&apos;t be opened because Apple cannot verify it.</q>
      </>
    ),
  },
  {
    title: "Choose Open Anyway",
    description: (
      <>
        Open <strong>System Settings</strong>, choose{" "}
        <strong>Privacy &amp; Security</strong>, scroll down, then select{" "}
        <strong>Open Anyway</strong>.
      </>
    ),
  },
];

const windowsSteps: InstallationStep[] = [
  {
    title: "Download installer",
    description: "Download the Windows x64 setup file.",
  },
  {
    title: "Launch setup",
    description: "Open the installer from your Downloads folder.",
  },
  {
    title: "SmartScreen appears",
    description:
      "Windows may show a Microsoft Defender SmartScreen warning.",
  },
  {
    title: "Run anyway",
    description: (
      <>
        Select <strong>More info</strong>, then choose{" "}
        <strong>Run anyway</strong> to continue.
      </>
    ),
  },
];

export function InstallationHelp() {
  return (
    <section
      aria-labelledby="installation-help-title"
      className="mt-14 sm:mt-16"
    >
      <motion.header
        initial={{ opacity: 0, y: 20 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true, amount: 0.35 }}
        transition={{ duration: 0.52, ease: [0.22, 1, 0.36, 1] }}
        className="max-w-[820px]"
      >
        <div className="flex items-center gap-3">
          <span className="grid size-11 shrink-0 place-items-center rounded-full border-2 border-ink bg-yellow shadow-brutal-sm">
            <CircleHelp aria-hidden="true" className="size-6" strokeWidth={2.6} />
          </span>
          <h3
            id="installation-help-title"
            className="text-3xl font-black tracking-[-0.045em] sm:text-4xl"
          >
            Need help installing?
          </h3>
        </div>
        <p className="mt-4 max-w-[760px] text-sm font-semibold leading-[1.75] text-ink/78 sm:text-base">
          Ducky is not code signed yet. Because of this, macOS Gatekeeper and
          Windows SmartScreen may show a warning. This is expected. Follow the
          steps below.
        </p>
      </motion.header>

      <div className="mt-7 grid min-w-0 grid-cols-1 gap-6 lg:grid-cols-2">
        <InstallationGuideCard
          title="macOS — Open Anyway"
          icon={AppleLogoIcon}
          steps={macSteps}
          illustration={<MacSecurityIllustration />}
          noteColor="bg-purple/14"
          note={
            <span className="flex items-start gap-2">
              <Lightbulb
                aria-hidden="true"
                className="mt-0.5 size-4 shrink-0"
                strokeWidth={2.5}
              />
              You only need to do this once. Future launches work normally.
            </span>
          }
          index={0}
        />

        <InstallationGuideCard
          title="Windows — Run Anyway"
          icon={Monitor}
          steps={windowsSteps}
          illustration={<WindowsSecurityIllustration />}
          noteColor="bg-mint/18"
          note={
            <>
              Microsoft shows this because Ducky is not code signed yet. It
              does not mean the application is unsafe.
            </>
          }
          index={1}
        />
      </div>

      <LinuxInstallCard />
    </section>
  );
}
