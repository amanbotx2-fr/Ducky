"use client";

import { Download, Star } from "lucide-react";
import { motion } from "framer-motion";
import { useEffect, useState, useSyncExternalStore } from "react";
import { supportLinks } from "../../lib/siteLinks";
import { BrandMark } from "../BrandMark";

const navigation = [
  { label: "Features", href: "#features", color: "bg-purple" },
  { label: "Download", href: "#download", color: "bg-orange" },
  { label: "Buy Me a Coffee", href: "#support", color: "bg-yellow" },
  { label: "FAQ", href: "#faq", color: "bg-purple" },
] as const;

type SectionId = (typeof navigation)[number]["href"] extends `#${infer Id}`
  ? Id
  : never;

const scrollThreshold = 20;

const subscribeToScrolledState = (onStoreChange: () => void) => {
  let wasScrolled = window.scrollY > scrollThreshold;

  const handleScroll = () => {
    const isScrolled = window.scrollY > scrollThreshold;

    if (isScrolled !== wasScrolled) {
      wasScrolled = isScrolled;
      onStoreChange();
    }
  };

  window.addEventListener("scroll", handleScroll, { passive: true });

  return () => window.removeEventListener("scroll", handleScroll);
};

const getScrolledSnapshot = () => window.scrollY > scrollThreshold;
const getServerScrolledSnapshot = () => false;

export function Navbar() {
  const [activeSection, setActiveSection] = useState<SectionId | null>(null);
  const isScrolled = useSyncExternalStore(
    subscribeToScrolledState,
    getScrolledSnapshot,
    getServerScrolledSnapshot,
  );

  useEffect(() => {
    const sections = navigation
      .map(({ href }) => document.getElementById(href.slice(1)))
      .filter((section): section is HTMLElement => section !== null);
    const intersectingSections = new Set<HTMLElement>();
    const activationRatio = 0.34;

    const updateActiveSection = () => {
      const activationY = window.innerHeight * activationRatio;
      const candidates = [...intersectingSections].sort(
        (first, second) =>
          Math.abs(first.getBoundingClientRect().top - activationY) -
          Math.abs(second.getBoundingClientRect().top - activationY),
      );
      const intersectingSection = candidates[0];

      if (intersectingSection) {
        setActiveSection(intersectingSection.id as SectionId);
        return;
      }

      const sectionAtActivationLine = sections.find((section) => {
        const bounds = section.getBoundingClientRect();
        return bounds.top <= activationY && bounds.bottom >= activationY;
      });

      if (sectionAtActivationLine) {
        setActiveSection(sectionAtActivationLine.id as SectionId);
        return;
      }

      const firstSection = sections[0];
      const lastSection = sections[sections.length - 1];

      if (
        firstSection &&
        firstSection.getBoundingClientRect().top > activationY
      ) {
        setActiveSection(null);
      } else if (
        lastSection &&
        lastSection.getBoundingClientRect().bottom < activationY
      ) {
        setActiveSection(lastSection.id as SectionId);
      }
    };

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          const section = entry.target as HTMLElement;

          if (entry.isIntersecting) {
            intersectingSections.add(section);
          } else {
            intersectingSections.delete(section);
          }
        });

        updateActiveSection();
      },
      {
        rootMargin: "-31% 0px -63% 0px",
        threshold: 0,
      },
    );

    sections.forEach((section) => observer.observe(section));

    return () => {
      observer.disconnect();
      intersectingSections.clear();
    };
  }, []);

  const selectSection = (href: `#${SectionId}`) => {
    setActiveSection(href.slice(1) as SectionId);
  };

  return (
    <>
      <div
        aria-hidden="true"
        className="h-[170px] sm:h-[122px] min-[1400px]:h-[62px]"
      />

      <div className="navbar-fixed-frame pointer-events-none fixed inset-x-0 top-[31px] z-50 sm:top-[43px] lg:top-[51px]">
        <motion.header
          initial={false}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.55, ease: [0.22, 1, 0.36, 1] }}
          data-scrolled={isScrolled}
          className="navbar-enter pointer-events-auto mx-auto w-[calc(100%-62px)] max-w-[1376px] sm:w-[calc(100%-94px)] lg:w-[calc(100%-118px)] xl:w-[calc(100%-134px)]"
        >
          <div className="flex items-center justify-between gap-4">
            <div
              className={`shrink-0 rounded-xl transition-[background-color,box-shadow,backdrop-filter] duration-300 ease-out ${
                isScrolled
                  ? "bg-[rgba(255,239,205,0.88)] shadow-[0_8px_24px_rgba(0,0,0,0.18)] backdrop-blur-[14px] [-webkit-backdrop-filter:blur(14px)]"
                  : "bg-transparent shadow-none"
              }`}
            >
              <BrandMark />
            </div>

            <nav
              aria-label="Primary navigation"
              className={`hidden items-center rounded-[15px] border-2 border-ink px-2 py-1.5 transition-[background-color,box-shadow,backdrop-filter] duration-300 ease-out min-[1400px]:flex ${
                isScrolled
                  ? "bg-[rgba(255,239,205,0.88)] shadow-[0_8px_24px_rgba(0,0,0,0.18),3px_3px_0_#111] backdrop-blur-[14px] [-webkit-backdrop-filter:blur(14px)]"
                  : "bg-[#FFD98A] shadow-brutal-sm"
              }`}
            >
              {navigation.map((item, index) => {
                const sectionId = item.href.slice(1) as SectionId;
                const isActive = activeSection === sectionId;

                return (
                  <a
                    key={item.label}
                    href={item.href}
                    onClick={() => selectSection(item.href)}
                    aria-current={isActive ? "location" : undefined}
                    className={`group flex min-h-11 items-center gap-2.5 rounded-lg px-4 py-2.5 text-sm font-extrabold transition-colors duration-200 hover:bg-[#FFE7AE] focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-orange/25 ${
                      isActive ? "bg-[#FFE7AE]" : ""
                    } ${
                      index !== navigation.length - 1
                        ? "after:ml-2 after:h-6 after:w-px after:bg-ink/35"
                        : ""
                    }`}
                  >
                    <span
                      aria-hidden="true"
                      className={`size-3 rounded-[3px] border-2 border-ink ${item.color}`}
                    />
                    {item.label}
                  </a>
                );
              })}
            </nav>

            <div className="flex shrink-0 items-center gap-3">
              <a
                href={supportLinks.repository}
                target="_blank"
                rel="noreferrer"
                className={`hidden h-12 items-center gap-2 rounded-xl border-2 border-ink px-4 text-sm font-black transition-[transform,background-color,box-shadow,backdrop-filter] duration-300 ease-out hover:-translate-y-1 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-yellow/45 lg:flex ${
                  isScrolled
                    ? "bg-[rgba(255,239,205,0.88)] shadow-[0_8px_24px_rgba(0,0,0,0.18),3px_3px_0_#111] backdrop-blur-[14px] [-webkit-backdrop-filter:blur(14px)]"
                    : "bg-cream shadow-brutal-sm"
                }`}
                aria-label="Star Ducky on GitHub"
              >
                <Star
                  aria-hidden="true"
                  className="size-5 fill-yellow text-ink"
                  strokeWidth={2.6}
                />
                Star on GitHub
              </a>
              <a
                href="#download"
                onClick={() => selectSection("#download")}
                className={`flex h-12 items-center gap-2 rounded-xl border-2 border-ink px-3.5 text-sm font-black transition-[transform,background-color,box-shadow,backdrop-filter] duration-300 ease-out hover:-translate-y-1 hover:shadow-brutal-lg focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-orange/30 sm:px-5 ${
                  isScrolled
                    ? "bg-[rgba(255,106,61,0.90)] shadow-[0_8px_24px_rgba(0,0,0,0.18),5px_5px_0_#111] backdrop-blur-[14px] [-webkit-backdrop-filter:blur(14px)]"
                    : "bg-orange shadow-brutal"
                }`}
              >
                <span className="hidden min-[460px]:inline">Download Now</span>
                <Download
                  aria-hidden="true"
                  className="size-5"
                  strokeWidth={2.7}
                />
              </a>
            </div>
          </div>

          <nav
            aria-label="Mobile navigation"
            className="mt-4 grid w-full min-w-0 grid-cols-2 gap-2 pb-1 sm:flex min-[1400px]:!hidden"
          >
            {navigation.map((item) => {
              const sectionId = item.href.slice(1) as SectionId;
              const isActive = activeSection === sectionId;

              return (
                <a
                  key={item.label}
                  href={item.href}
                  onClick={() => selectSection(item.href)}
                  aria-current={isActive ? "location" : undefined}
                  className={`flex min-h-11 w-full shrink-0 items-center justify-center gap-2 whitespace-nowrap rounded-[11px] border-2 border-ink px-2.5 text-[0.7rem] font-extrabold transition-[background-color,box-shadow,backdrop-filter] duration-300 ease-out focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-orange/25 sm:w-auto sm:px-3.5 sm:text-xs ${
                    isScrolled
                      ? `${
                          isActive
                            ? "bg-[rgba(255,231,174,0.92)]"
                            : "bg-[rgba(255,239,205,0.88)]"
                        } shadow-[0_8px_24px_rgba(0,0,0,0.18),3px_3px_0_#111] backdrop-blur-[14px] [-webkit-backdrop-filter:blur(14px)]`
                      : `${
                          isActive ? "bg-[#FFE7AE]" : "bg-[#FFD98A]"
                        } shadow-brutal-sm`
                  }`}
                >
                  <span
                    aria-hidden="true"
                    className={`size-2.5 rounded-[2px] border border-ink ${item.color}`}
                  />
                  {item.label}
                </a>
              );
            })}
          </nav>
        </motion.header>
      </div>
    </>
  );
}
