import {
  Diamond,
  Map,
  MessageCircleQuestion,
  Sparkles,
  Star,
  type LucideIcon,
} from "lucide-react";
import { SectionContainer } from "../SectionContainer";

type ComingSoonSectionProps = {
  id: "roadmap" | "faq";
  title: string;
  icon: LucideIcon;
  accent: string;
};

function ComingSoonSection({
  id,
  title,
  icon: Icon,
  accent,
}: ComingSoonSectionProps) {
  const titleId = `${id}-title`;

  return (
    <section
      id={id}
      aria-labelledby={titleId}
      className="landing-section-anchor relative overflow-hidden bg-cream pb-12 pt-2 sm:pb-16 sm:pt-4"
    >
      <SectionContainer>
        <div className="relative grid min-h-[520px] place-items-center overflow-hidden rounded-[26px] border-[3px] border-ink bg-cream px-5 py-14 text-center shadow-brutal-shell sm:min-h-[580px]">
          <div className="relative z-10">
            <span
              className={`mx-auto grid size-16 place-items-center rounded-[18px] border-[3px] border-ink shadow-brutal ${accent}`}
            >
              <Icon aria-hidden="true" className="size-8" strokeWidth={2.4} />
            </span>
            <h2
              id={titleId}
              className="mt-7 text-[clamp(2.7rem,6vw,4.6rem)] font-black leading-none tracking-[-0.065em]"
            >
              {title}
            </h2>
            <p className="mt-4 text-base font-bold text-ink/68 sm:text-lg">
              Coming soon.
            </p>
          </div>

          <Star
            aria-hidden="true"
            className="absolute left-[12%] top-[24%] hidden size-7 fill-yellow text-ink sm:block"
            strokeWidth={2.2}
          />
          <Diamond
            aria-hidden="true"
            className="absolute right-[13%] top-[22%] hidden size-5 rotate-12 fill-pink text-ink sm:block"
            strokeWidth={2.2}
          />
          <Sparkles
            aria-hidden="true"
            className="absolute bottom-[18%] right-[20%] size-6 text-orange"
            strokeWidth={2.3}
          />
        </div>
      </SectionContainer>
    </section>
  );
}

export function FutureSections() {
  return (
    <>
      <ComingSoonSection
        id="roadmap"
        title="Roadmap"
        icon={Map}
        accent="bg-yellow"
      />
      <ComingSoonSection
        id="faq"
        title="Frequently Asked Questions"
        icon={MessageCircleQuestion}
        accent="bg-purple"
      />
    </>
  );
}
