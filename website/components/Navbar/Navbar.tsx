"use client";

import { Download, Star } from "lucide-react";
import { motion } from "framer-motion";
import { useEffect, useState } from "react";
import { BrandMark } from "../BrandMark";

const navigation = [
  { label: "Features", href: "#features", color: "bg-purple" },
  { label: "Download", href: "#download", color: "bg-orange" },
  { label: "Roadmap", href: "#roadmap", color: "bg-yellow" },
  { label: "FAQ", href: "#faq", color: "bg-purple" },
] as const;

type SectionId = (typeof navigation)[number]["href"] extends `#${infer Id}`
  ? Id
  : never;

const repositoryUrl = "https://github.com/amanbotx2-fr/Ducky";

export function Navbar() {
  const [activeSection, setActiveSection] = useState<SectionId | null>(null);

  useEffect(() => {
    const sectionEntries = new Map<Element, IntersectionObserverEntry>();
    const sections = navigation
      .map(({ href }) => document.getElementById(href.slice(1)))
      .filter((section): section is HTMLElement => section !== null);

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => sectionEntries.set(entry.target, entry));

        const visibleSections = [...sectionEntries.values()]
          .filter((entry) => entry.isIntersecting)
          .sort(
            (first, second) =>
              Math.abs(first.boundingClientRect.top - window.innerHeight * 0.63) -
              Math.abs(second.boundingClientRect.top - window.innerHeight * 0.63),
          );

        const currentSection = visibleSections[0]?.target.id as
          | SectionId
          | undefined;

        if (currentSection) {
          setActiveSection(currentSection);
        }
      },
      {
        rootMargin: "-58% 0px -32% 0px",
        threshold: 0,
      },
    );

    sections.forEach((section) => observer.observe(section));

    return () => observer.disconnect();
  }, []);

  const selectSection = (href: `#${SectionId}`) => {
    setActiveSection(href.slice(1) as SectionId);
  };

  return (
    <>
      <div
        aria-hidden="true"
        className="h-[108px] sm:h-[122px] min-[1400px]:h-[62px]"
      />

      <div className="pointer-events-none fixed inset-x-0 top-[31px] z-50 sm:top-[43px] lg:top-[51px]">
        <motion.header
          initial={false}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.55, ease: [0.22, 1, 0.36, 1] }}
          className="navbar-enter pointer-events-auto mx-auto w-[calc(100%-62px)] max-w-[1376px] rounded-[18px] bg-cream sm:w-[calc(100%-94px)] lg:w-[calc(100%-118px)] xl:w-[calc(100%-134px)]"
        >
          <div className="flex items-center justify-between gap-4">
            <BrandMark />

            <nav
              aria-label="Primary navigation"
              className="hidden items-center rounded-[15px] border-2 border-ink bg-cream px-2 py-1.5 shadow-brutal-sm min-[1400px]:flex"
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
                    className={`group flex items-center gap-2.5 rounded-lg px-4 py-2.5 text-sm font-extrabold transition-colors duration-200 hover:bg-yellow/30 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-orange/25 ${
                      isActive ? "bg-yellow/55" : ""
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
                href={repositoryUrl}
                target="_blank"
                rel="noreferrer"
                className="hidden h-12 items-center gap-2 rounded-xl border-2 border-ink bg-cream px-4 text-sm font-black shadow-brutal-sm transition-transform hover:-translate-y-1 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-yellow/45 lg:flex"
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
                className="flex h-12 items-center gap-2 rounded-xl border-2 border-ink bg-orange px-3.5 text-sm font-black shadow-brutal transition-transform hover:-translate-y-1 hover:shadow-brutal-lg focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-orange/30 sm:px-5"
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
            className="scrollbar-none mt-4 flex w-full min-w-0 snap-x gap-2 overflow-x-auto pb-1 min-[1400px]:hidden"
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
                  className={`flex h-10 shrink-0 snap-start items-center gap-2 rounded-[11px] border-2 border-ink px-3.5 text-xs font-extrabold shadow-brutal-sm transition-colors duration-200 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-orange/25 ${
                    isActive ? "bg-yellow" : "bg-cream"
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
