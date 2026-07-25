import type { ReactNode } from "react";

type SectionContainerProps = {
  children: ReactNode;
  className?: string;
};

export function SectionContainer({
  children,
  className = "",
}: SectionContainerProps) {
  return (
    <div
      className={`mx-auto w-full max-w-[1510px] px-3 sm:px-5 lg:px-6 ${className}`}
    >
      {children}
    </div>
  );
}
