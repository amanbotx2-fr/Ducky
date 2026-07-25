import { DownloadSection } from "../components/Download/DownloadSection";
import { FutureSections } from "../components/ComingSoon/FutureSections";
import { FeaturesSection } from "../components/Features/FeaturesSection";
import { Hero } from "../components/Hero/Hero";

export default function Home() {
  return (
    <main>
      <Hero />
      <FeaturesSection />
      <DownloadSection />
      <FutureSections />
    </main>
  );
}
