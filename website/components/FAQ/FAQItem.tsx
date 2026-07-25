"use client";

import { ChevronDown, type LucideIcon } from "lucide-react";
import { useId, useState, type ReactNode } from "react";

type FAQItemProps = {
  question: string;
  answer: ReactNode;
  icon: LucideIcon;
  color: string;
};

export function FAQItem({
  question,
  answer,
  icon: Icon,
  color,
}: FAQItemProps) {
  const [isOpen, setIsOpen] = useState(false);
  const id = useId();
  const panelId = `faq-answer-${id}`;
  const buttonId = `faq-question-${id}`;

  return (
    <article className="min-w-0 rounded-[19px] border-[3px] border-ink bg-cream shadow-brutal transition-[transform,box-shadow] duration-300 hover:-translate-y-1 hover:shadow-brutal-lg">
      <button
        id={buttonId}
        type="button"
        aria-expanded={isOpen}
        aria-controls={panelId}
        onClick={() => setIsOpen((open) => !open)}
        className="group flex min-h-[118px] w-full items-center gap-4 rounded-[16px] p-4 text-left outline-none focus-visible:ring-4 focus-visible:ring-inset focus-visible:ring-orange/35 sm:p-5"
      >
        <span
          className={`grid size-16 shrink-0 place-items-center rounded-[15px] border-[3px] border-ink shadow-brutal-sm ${color}`}
        >
          <Icon aria-hidden="true" className="size-8" strokeWidth={2.35} />
        </span>
        <span className="min-w-0 flex-1 text-base font-black leading-snug tracking-[-0.025em] sm:text-lg">
          {question}
        </span>
        <ChevronDown
          aria-hidden="true"
          className={`size-6 shrink-0 transition-transform duration-[225ms] ${
            isOpen ? "rotate-180" : ""
          }`}
          strokeWidth={2.7}
        />
      </button>

      <div
        className={`grid transition-[grid-template-rows,opacity] duration-[225ms] ease-out ${
          isOpen
            ? "grid-rows-[1fr] opacity-100"
            : "grid-rows-[0fr] opacity-0"
        }`}
      >
        <div className="overflow-hidden">
          <div
            id={panelId}
            role="region"
            aria-labelledby={buttonId}
            aria-hidden={!isOpen}
            className="px-5 pb-5 pl-[100px] text-sm font-semibold leading-[1.7] text-ink/74 sm:pl-[108px] sm:text-[0.95rem]"
          >
            {answer}
          </div>
        </div>
      </div>
    </article>
  );
}
