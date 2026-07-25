import { AppWindow, TerminalSquare } from "lucide-react";
import { downloadLinks } from "../../lib/siteLinks";
import { AppleLogoIcon } from "../icons/AppleLogoIcon";
import { PlatformCard, type PlatformDownload } from "./PlatformCard";

const platformDownloads: PlatformDownload[] = [
  {
    title: "macOS",
    subtitle: "macOS 10.14+",
    detail: "Intel and Apple Silicon",
    buttonLabel: "Download for macOS",
    href: downloadLinks.mac,
    icon: AppleLogoIcon,
    iconColor: "bg-purple",
    chips: ["Apple Silicon", "Intel x64"],
    rotation: -0.18,
  },
  {
    title: "Windows",
    subtitle: "Windows 10 / 11",
    detail: "64-bit installer",
    buttonLabel: "Download for Windows",
    href: downloadLinks.windows,
    icon: AppWindow,
    iconColor: "bg-mint",
    chips: ["x64 Installer"],
    rotation: 0.16,
  },
  {
    title: "Linux",
    subtitle: "Ubuntu, Debian, Fedora",
    detail: "Arch and other modern distributions",
    buttonLabel: "Download AppImage",
    href: downloadLinks.linux,
    icon: TerminalSquare,
    iconColor: "bg-yellow",
    chips: ["Portable"],
    rotation: -0.12,
  },
];

export function PlatformDownloads() {
  return (
    <section aria-labelledby="platform-downloads-title" className="mt-10">
      <h3 id="platform-downloads-title" className="sr-only">
        Choose your Ducky download
      </h3>

      <div className="grid min-w-0 grid-cols-1 gap-5 md:grid-cols-2 lg:grid-cols-3">
        {platformDownloads.map((platform, index) => (
          <PlatformCard key={platform.title} {...platform} index={index} />
        ))}
      </div>
    </section>
  );
}
