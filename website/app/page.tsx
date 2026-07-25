import { DownloadSection } from "../components/Download/DownloadSection";
import { FaqSection } from "../components/FAQ/FAQSection";
import { FeaturesSection } from "../components/Features/FeaturesSection";
import { Hero } from "../components/Hero/Hero";
import { SupportSection } from "../components/Support/SupportSection";

export default function Home() {
  return (
    <main>
      <Hero />
      <FeaturesSection />
      <DownloadSection />
      <SupportSection />
      <FaqSection />
    </main>
  );
}
