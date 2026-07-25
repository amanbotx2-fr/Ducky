import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

async function render() {
  const workerUrl = new URL("../dist/server/index.js", import.meta.url);
  workerUrl.searchParams.set("test", `${process.pid}-${Date.now()}`);
  const { default: worker } = await import(workerUrl.href);

  return worker.fetch(
    new Request("http://localhost/", {
      headers: { accept: "text/html" },
    }),
    {
      ASSETS: {
        fetch: async () => new Response("Not found", { status: 404 }),
      },
    },
    {
      waitUntil() {},
      passThroughOnException() {},
    },
  );
}

test("server-renders the complete one-page Ducky landing page", async () => {
  const response = await render();
  assert.equal(response.status, 200);
  assert.match(response.headers.get("content-type") ?? "", /^text\/html\b/i);

  const html = await response.text();
  assert.match(html, /<title>Ducky — Your Desktop\. Your Buddy\.<\/title>/i);
  assert.match(html, /Your Desktop\./);
  assert.match(html, /Your Buddy\./);
  assert.match(html, /That(?:&apos;|&#x27;|')s Ducky\./);
  assert.match(html, /AI Desktop Companion/);
  assert.match(html, /Notification Sounds/);
  assert.match(html, /Everything you need\./);
  assert.match(html, /Right on your/);
  assert.match(html, /AI Model Explorer/);
  assert.match(html, /Native Desktop/);
  assert.match(html, /I(?:&apos;|&#x27;|')m always here\./);
  assert.match(html, /Built for productivity\. Designed for everyone\./);
  assert.match(html, /Download Ducky\./);
  assert.match(html, /Bring your/);
  assert.match(html, /Download for macOS/);
  assert.match(html, /Download for Windows/);
  assert.match(html, /Download AppImage/);
  assert.match(html, /Ducky-1\.1\.0-universal\.dmg/);
  assert.match(html, /Ducky-Setup-1\.1\.0-x64\.exe/);
  assert.match(html, /Ducky-1\.1\.0-x86_64\.AppImage/);
  assert.match(html, /Need help installing\?/);
  assert.match(html, /macOS — Open Anyway/);
  assert.match(html, /Windows — Run Anyway/);
  assert.match(html, /Linux — AppImage/);
  assert.match(html, /Still need help\?/);
  assert.match(html, /Download with confidence\./);
  assert.match(html, /I(?:&apos;|&#x27;|')ll be waiting\./);
  assert.match(html, /Roadmap/);
  assert.match(html, /Frequently Asked Questions/);
  assert.match(html, /Coming soon\./);
  assert.match(html, /id="features"/);
  assert.match(html, /id="download"/);
  assert.match(html, /id="roadmap"/);
  assert.match(html, /id="faq"/);
  assert.match(html, /official pixel-art desktop companion mascot/);
  assert.doesNotMatch(html, /codex-preview|SkeletonPreview|Starter Project/i);
});

test("keeps the landing page scoped to the requested sections", async () => {
  const [
    page,
    hero,
    navbar,
    mascotWindow,
    heroButtons,
    featureStrip,
    featuresSection,
    featureCard,
    downloadSection,
    platformDownloads,
    installationHelp,
    futureSections,
    globals,
    releaseAssets,
    brandAssets,
  ] = await Promise.all([
    readFile(new URL("../app/page.tsx", import.meta.url), "utf8"),
    readFile(new URL("../components/Hero/Hero.tsx", import.meta.url), "utf8"),
    readFile(
      new URL("../components/Navbar/Navbar.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/Hero/MascotWindow.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/Hero/HeroButtons.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/Hero/FeatureStrip.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/Features/FeaturesSection.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/Features/FeatureCard.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/Download/DownloadSection.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/Download/PlatformDownloads.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/Download/InstallationHelp.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/ComingSoon/FutureSections.tsx", import.meta.url),
      "utf8",
    ),
    readFile(new URL("../app/globals.css", import.meta.url), "utf8"),
    readFile(new URL("../lib/releaseAssets.ts", import.meta.url), "utf8"),
    readFile(new URL("../lib/brandAssets.ts", import.meta.url), "utf8"),
  ]);

  assert.match(page, /<Hero \/>/);
  assert.match(page, /<FeaturesSection \/>/);
  assert.match(page, /<DownloadSection \/>/);
  assert.match(page, /<FutureSections \/>/);
  assert.ok(page.indexOf("<Hero />") < page.indexOf("<FeaturesSection />"));
  assert.ok(
    page.indexOf("<FeaturesSection />") < page.indexOf("<DownloadSection />"),
  );
  assert.ok(
    page.indexOf("<DownloadSection />") < page.indexOf("<FutureSections />"),
  );
  assert.doesNotMatch(page, /<section|<footer/i);
  assert.match(hero, /<Navbar \/>/);
  assert.match(hero, /<MascotWindow \/>/);
  assert.match(hero, /<FeatureStrip \/>/);
  assert.ok(navbar.indexOf('label: "Features"') < navbar.indexOf('label: "Download"'));
  assert.ok(navbar.indexOf('label: "Download"') < navbar.indexOf('label: "Roadmap"'));
  assert.ok(navbar.indexOf('label: "Roadmap"') < navbar.indexOf('label: "FAQ"'));
  assert.match(navbar, /new IntersectionObserver/);
  assert.match(navbar, /aria-current=/);
  assert.match(navbar, /href="#download"/);
  assert.match(navbar, /https:\/\/github\.com\/amanbotx2-fr\/Ducky/);
  assert.match(heroButtons, /releases\/latest/);
  assert.doesNotMatch(heroButtons, /id="download"/);
  assert.match(mascotWindow, /brandAssets/);
  assert.match(brandAssets, /mascot pic\/master\.png/);
  assert.match(featureStrip, /Notification Sounds/);
  assert.doesNotMatch(featureStrip, /id="features"/);
  assert.doesNotMatch(featureStrip, /Eye Tracking/i);
  assert.match(featuresSection, /FeatureCard/);
  assert.match(featuresSection, /FeaturesMascotPanel/);
  assert.match(featuresSection, /CapabilityStrip/);
  assert.match(featuresSection, /id="features"/);
  assert.match(featureCard, /focus-visible:ring/);
  assert.match(downloadSection, /DownloadHeader/);
  assert.match(downloadSection, /PlatformDownloads/);
  assert.match(downloadSection, /InstallationHelp/);
  assert.match(downloadSection, /SupportSection/);
  assert.match(downloadSection, /CapabilityStrip/);
  assert.match(downloadSection, /id="download"/);
  assert.match(platformDownloads, /releaseAssets\.macos/);
  assert.match(platformDownloads, /releaseAssets\.windows/);
  assert.match(platformDownloads, /releaseAssets\.linux/);
  assert.match(installationHelp, /Gatekeeper/);
  assert.match(installationHelp, /SmartScreen/);
  assert.match(futureSections, /id="roadmap"/);
  assert.match(futureSections, /id="faq"/);
  assert.match(futureSections, /Frequently Asked Questions/);
  assert.match(globals, /scroll-behavior:\s*smooth/);
  assert.match(globals, /\.landing-section-anchor/);
  assert.match(
    globals,
    /@media \(prefers-reduced-motion: reduce\)[\s\S]*scroll-behavior:\s*auto/,
  );
  assert.match(releaseAssets, /const releaseVersion = "1\.1\.0"/);
  assert.match(releaseAssets, /Ducky-\$\{releaseVersion\}-universal\.dmg/);
  assert.match(releaseAssets, /Ducky-Setup-\$\{releaseVersion\}-x64\.exe/);
  assert.match(releaseAssets, /Ducky-\$\{releaseVersion\}-x86_64\.AppImage/);
});
