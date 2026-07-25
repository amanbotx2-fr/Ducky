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
        className="group flex min-h-[104px] w-full items-center gap-3 rounded-[16px] p-3 text-left outline-none focus-visible:ring-4 focus-visible:ring-inset focus-visible:ring-orange/35 min-[380px]:min-h-[112px] min-[380px]:p-4 sm:min-h-[118px] sm:gap-4 sm:p-5"
      >
        <span
          className={`grid size-14 shrink-0 place-items-center rounded-[14px] border-[3px] border-ink shadow-brutal-sm min-[380px]:size-[60px] sm:size-16 sm:rounded-[15px] ${color}`}
        >
          <Icon
            aria-hidden="true"
            className="size-7 min-[380px]:size-[30px] sm:size-8"
            strokeWidth={2.35}
          />
        </span>
        <span className="min-w-0 flex-1 text-[0.95rem] font-black leading-snug tracking-[-0.025em] min-[380px]:text-base sm:text-lg">
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
            className="px-4 pb-4 text-[0.82rem] font-semibold leading-[1.7] text-ink/74 sm:px-5 sm:pb-5 sm:pl-[108px] sm:text-[0.95rem]"
          >
            {answer}
          </div>
        </div>
      </div>
    </article>
  );
}
