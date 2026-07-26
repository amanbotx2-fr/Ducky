import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { readFile } from "node:fs/promises";
import { createServer } from "node:net";
import test, { after, before } from "node:test";
import { fileURLToPath } from "node:url";

const projectRoot = fileURLToPath(new URL("..", import.meta.url));
let nextServer;
let serverOrigin;

async function getAvailablePort() {
  const server = createServer();
  server.listen(0, "127.0.0.1");
  await once(server, "listening");

  const address = server.address();
  assert.ok(address && typeof address === "object");
  const { port } = address;

  server.close();
  await once(server, "close");
  return port;
}

async function waitForServer(origin, output) {
  const deadline = Date.now() + 30_000;

  while (Date.now() < deadline) {
    if (nextServer.exitCode !== null) {
      throw new Error(
        `Next.js exited before becoming ready.\n${output.join("")}`,
      );
    }

    try {
      const response = await fetch(origin, {
        headers: { accept: "text/html" },
      });
      if (response.ok) {
        return;
      }
    } catch {
      // The production server is still starting.
    }

    await new Promise((resolve) => setTimeout(resolve, 100));
  }

  throw new Error(`Timed out waiting for Next.js.\n${output.join("")}`);
}

before(async () => {
  const port = await getAvailablePort();
  const output = [];
  serverOrigin = `http://127.0.0.1:${port}`;
  nextServer = spawn(
    process.execPath,
    ["node_modules/next/dist/bin/next", "start", "--hostname", "127.0.0.1", "--port", String(port)],
    {
      cwd: projectRoot,
      env: { ...process.env, NODE_ENV: "production" },
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  nextServer.stdout.on("data", (chunk) => output.push(chunk.toString()));
  nextServer.stderr.on("data", (chunk) => output.push(chunk.toString()));
  await waitForServer(serverOrigin, output);
});

after(async () => {
  if (!nextServer || nextServer.exitCode !== null) {
    return;
  }

  nextServer.kill("SIGTERM");
  await Promise.race([
    once(nextServer, "exit"),
    new Promise((resolve) => setTimeout(resolve, 5_000)),
  ]);
});

async function render(pathname = "/") {
  return fetch(new URL(pathname, serverOrigin), {
    headers: { accept: "text/html" },
  });
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
  assert.match(html, /href="\/download\/mac"/);
  assert.match(html, /href="\/download\/windows"/);
  assert.match(html, /href="\/download\/linux"/);
  assert.equal(
    (html.match(/data-platform-icon="apple"/g) ?? []).length,
    3,
  );
  assert.match(html, /Ducky-\*\.AppImage/);
  assert.doesNotMatch(
    html,
    /releases\/download|Ducky-\d+\.\d+\.\d+/,
  );
  assert.match(html, /Need help installing\?/);
  assert.match(html, /macOS — Open Anyway/);
  assert.match(html, /Windows — Run Anyway/);
  assert.match(html, /Linux — AppImage/);
  assert.match(html, /Still need help\?/);
  assert.match(html, /Download with confidence\./);
  assert.match(html, /I(?:&apos;|&#x27;|')ll be waiting\./);
  assert.match(html, /Buy Ducky a Coffee\./);
  assert.match(html, /Fuel/);
  assert.match(html, /more features\./);
  assert.match(html, /Support Ducky/);
  assert.match(html, /Buy Me a Coffee/);
  assert.doesNotMatch(html, /GitHub Sponsors/);
  assert.doesNotMatch(html, /Support via UPI/);
  assert.match(html, /Open Source Forever/);
  assert.match(html, /Thanks a latte!/);
  assert.match(html, /Still curious\?/);
  assert.match(html, /We(?:&apos;|&#x27;|')ve got you\./);
  assert.match(html, /Ask away!/);
  assert.match(html, /Is Ducky really free\?/);
  assert.match(html, /Does Ducky collect my data\?/);
  assert.match(html, /Which AI providers are supported\?/);
  assert.match(html, /Why does Windows show SmartScreen\?/);
  assert.match(html, /Can I contribute\?/);
  assert.match(html, /Ready to bring Ducky/);
  assert.match(html, /aria-expanded="false"/);
  assert.doesNotMatch(html, /Coming soon\./);
  assert.match(html, /id="features"/);
  assert.match(html, /id="download"/);
  assert.match(html, /id="support"/);
  assert.match(html, /id="faq"/);
  assert.match(html, /official pixel-art desktop companion mascot/);
  assert.doesNotMatch(html, /codex-preview|SkeletonPreview|Starter Project/i);
});

test("keeps the landing page scoped to the requested sections", async () => {
  const [
    layout,
    page,
    hero,
    navbar,
    mascotWindow,
    heroButtons,
    appleLogoIcon,
    featureStrip,
    featuresSection,
    featureCard,
    downloadSection,
    platformDownloads,
    installationHelp,
    linuxInstallCard,
    supportSection,
    supportHero,
    supportCards,
    supportBenefits,
    faqSection,
    faqHero,
    faqAccordion,
    faqItem,
    helpCards,
    finalCta,
    globals,
    siteLinks,
    githubRelease,
    downloadTracker,
    routeHandler,
    macRoute,
    windowsRoute,
    linuxRoute,
    brandAssets,
  ] = await Promise.all([
    readFile(new URL("../app/layout.tsx", import.meta.url), "utf8"),
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
      new URL("../components/icons/AppleLogoIcon.tsx", import.meta.url),
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
      new URL("../components/Download/LinuxInstallCard.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/Support/SupportSection.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/Support/SupportHero.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/Support/SupportCards.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/Support/SupportBenefits.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/FAQ/FAQSection.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/FAQ/FAQHero.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/FAQ/FAQAccordion.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/FAQ/FAQItem.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/FAQ/HelpCards.tsx", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../components/FAQ/FinalCTA.tsx", import.meta.url),
      "utf8",
    ),
    readFile(new URL("../app/globals.css", import.meta.url), "utf8"),
    readFile(new URL("../lib/siteLinks.ts", import.meta.url), "utf8"),
    readFile(
      new URL("../lib/downloads/githubRelease.ts", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../lib/downloads/downloadTracker.ts", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../lib/downloads/routeHandler.ts", import.meta.url),
      "utf8",
    ),
    readFile(new URL("../app/download/mac/route.ts", import.meta.url), "utf8"),
    readFile(
      new URL("../app/download/windows/route.ts", import.meta.url),
      "utf8",
    ),
    readFile(
      new URL("../app/download/linux/route.ts", import.meta.url),
      "utf8",
    ),
    readFile(new URL("../lib/brandAssets.ts", import.meta.url), "utf8"),
  ]);

  assert.match(layout, /from "@vercel\/analytics\/next"/);
  assert.match(layout, /<Analytics \/>/);
  assert.match(page, /<Hero \/>/);
  assert.match(page, /<FeaturesSection \/>/);
  assert.match(page, /<DownloadSection \/>/);
  assert.match(page, /<SupportSection \/>/);
  assert.match(page, /<FaqSection \/>/);
  assert.ok(page.indexOf("<Hero />") < page.indexOf("<FeaturesSection />"));
  assert.ok(
    page.indexOf("<FeaturesSection />") < page.indexOf("<DownloadSection />"),
  );
  assert.ok(
    page.indexOf("<DownloadSection />") < page.indexOf("<SupportSection />"),
  );
  assert.ok(
    page.indexOf("<SupportSection />") < page.indexOf("<FaqSection />"),
  );
  assert.doesNotMatch(page, /<section|<footer/i);
  assert.match(hero, /<Navbar \/>/);
  assert.match(hero, /<MascotWindow \/>/);
  assert.match(hero, /<FeatureStrip \/>/);
  assert.ok(navbar.indexOf('label: "Features"') < navbar.indexOf('label: "Download"'));
  assert.ok(
    navbar.indexOf('label: "Download"') <
      navbar.indexOf('label: "Buy Me a Coffee"'),
  );
  assert.ok(
    navbar.indexOf('label: "Buy Me a Coffee"') <
      navbar.indexOf('label: "FAQ"'),
  );
  assert.match(navbar, /new IntersectionObserver/);
  assert.match(navbar, /intersectingSections/);
  assert.match(navbar, /activationRatio = 0\.34/);
  assert.match(navbar, /getBoundingClientRect/);
  assert.match(navbar, /rootMargin: "-31% 0px -63% 0px"/);
  assert.doesNotMatch(navbar, /Map<Element, IntersectionObserverEntry>/);
  assert.match(navbar, /scrollThreshold = 20/);
  assert.match(navbar, /useSyncExternalStore/);
  assert.match(navbar, /rgba\(255,239,205,0\.88\)/);
  assert.match(navbar, /backdrop-blur-\[14px\]/);
  assert.match(navbar, /rgba\(255,106,61,0\.90\)/);
  assert.doesNotMatch(navbar, /rgba\(255,249,239,0\.97\)/);
  assert.match(navbar, /grid-cols-2/);
  assert.match(navbar, /min-h-11/);
  assert.match(navbar, /min-\[1400px\]:!hidden/);
  assert.doesNotMatch(navbar, /overflow-x-auto/);
  assert.match(navbar, /aria-current=/);
  assert.match(navbar, /href="#download"/);
  assert.match(navbar, /supportLinks\.repository/);
  assert.doesNotMatch(navbar, /https:\/\/github\.com/);
  assert.match(heroButtons, /downloadLinks\.mac/);
  assert.match(heroButtons, /downloadLinks\.windows/);
  assert.match(heroButtons, /downloadLinks\.linux/);
  assert.doesNotMatch(heroButtons, /https:\/\/github\.com|releases\/latest/);
  assert.match(heroButtons, /AppleLogoIcon/);
  assert.doesNotMatch(heroButtons, /import \{[^}]*\bApple\b[^}]*\}/);
  assert.match(appleLogoIcon, /viewBox="0 0 24 24"/);
  assert.match(appleLogoIcon, /fill="#111111"/);
  assert.match(appleLogoIcon, /data-platform-icon="apple"/);
  assert.doesNotMatch(heroButtons, /id="download"/);
  assert.match(mascotWindow, /brandAssets/);
  assert.match(brandAssets, /mascot pic\/master\.png/);
  assert.match(brandAssets, /assets\/support\/ducky-coffee\.png/);
  assert.match(brandAssets, /assets\/faq\/ducky-faq\.png/);
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
  assert.match(platformDownloads, /downloadLinks\.mac/);
  assert.match(platformDownloads, /downloadLinks\.windows/);
  assert.match(platformDownloads, /downloadLinks\.linux/);
  assert.match(platformDownloads, /AppleLogoIcon/);
  assert.doesNotMatch(platformDownloads, /import \{[^}]*\bApple\b[^}]*\}/);
  assert.match(platformDownloads, /lg:grid-cols-3/);
  assert.match(installationHelp, /Gatekeeper/);
  assert.match(installationHelp, /SmartScreen/);
  assert.match(installationHelp, /AppleLogoIcon/);
  assert.doesNotMatch(installationHelp, /import \{[^}]*\bApple\b[^}]*\}/);
  assert.match(installationHelp, /lg:grid-cols-2/);
  assert.match(linuxInstallCard, /Ducky-\*\.AppImage/);
  assert.match(linuxInstallCard, /downloadLinks\.linux/);
  assert.doesNotMatch(linuxInstallCard, /releaseAssets|\d+\.\d+\.\d+/);
  assert.match(supportSection, /id="support"/);
  assert.match(supportSection, /SupportHero/);
  assert.match(supportSection, /SupportCards/);
  assert.match(supportSection, /SupportBenefits/);
  assert.match(supportHero, /duckyCoffee/);
  assert.match(supportHero, /Thanks a latte|SupportSpeechBubble/);
  assert.match(supportCards, /NEXT_PUBLIC_BUY_ME_A_COFFEE_URL/);
  assert.doesNotMatch(supportCards, /NEXT_PUBLIC_GITHUB_SPONSORS_URL/);
  assert.doesNotMatch(supportCards, /NEXT_PUBLIC_UPI_SUPPORT_URL/);
  assert.match(supportCards, /md:grid-cols-\[minmax\(0,2fr\)_minmax\(220px,1fr\)\]/);
  assert.match(supportCards, /aria-disabled="true"/);
  assert.match(supportBenefits, /CapabilityStrip/);
  assert.match(supportBenefits, /Open Source Forever/);
  assert.match(faqSection, /id="faq"/);
  assert.match(faqSection, /FAQHero/);
  assert.match(faqSection, /FAQAccordion/);
  assert.match(faqSection, /HelpCards/);
  assert.match(faqSection, /FinalCTA/);
  assert.match(faqHero, /duckyFaq/);
  assert.match(faqHero, /Ask away!/);
  assert.match(faqHero, /Privacy First/);
  assert.match(faqAccordion, /Is Ducky really free\?/);
  assert.match(faqAccordion, /Does Ducky collect my data\?/);
  assert.match(faqAccordion, /OpenAI-compatible endpoints/);
  assert.match(faqAccordion, /Can I contribute\?/);
  assert.match(faqItem, /aria-expanded=/);
  assert.match(faqItem, /aria-controls=/);
  assert.match(faqItem, /aria-hidden=\{!isOpen\}/);
  assert.match(faqItem, /role="region"/);
  assert.match(faqItem, /duration-\[225ms\]/);
  assert.match(faqItem, /grid-rows-\[0fr\]/);
  assert.match(faqItem, /focus-visible:ring/);
  assert.match(helpCards, /Documentation/);
  assert.match(helpCards, /GitHub Issues/);
  assert.match(helpCards, /NEXT_PUBLIC_BUY_ME_A_COFFEE_URL/);
  assert.match(finalCta, /Ready to bring Ducky/);
  assert.match(finalCta, /href: "#download"/);
  assert.match(finalCta, /supportLinks\.repository/);
  assert.match(finalCta, /NEXT_PUBLIC_BUY_ME_A_COFFEE_URL/);
  assert.doesNotMatch(faqSection, /Coming soon/i);
  assert.match(globals, /scroll-behavior:\s*smooth/);
  assert.match(globals, /\.landing-section-anchor/);
  assert.match(globals, /scroll-margin-top:\s*14\.75rem/);
  assert.match(globals, /\.navbar-fixed-frame/);
  assert.match(
    globals,
    /@media \(prefers-reduced-motion: reduce\)[\s\S]*scroll-behavior:\s*auto/,
  );
  assert.match(siteLinks, /mac: "\/download\/mac"/);
  assert.match(siteLinks, /windows: "\/download\/windows"/);
  assert.match(siteLinks, /linux: "\/download\/linux"/);
  assert.doesNotMatch(
    siteLinks,
    /releases\/download|releaseVersion|\d+\.\d+\.\d+/,
  );
  assert.match(githubRelease, /releases\/latest/);
  assert.match(githubRelease, /next:\s*\{\s*revalidate:/);
  assert.match(githubRelease, /\.dmg/);
  assert.match(githubRelease, /\.exe/);
  assert.match(githubRelease, /\.appimage/);
  assert.match(downloadTracker, /interface DownloadTracker/);
  assert.match(downloadTracker, /class SupabaseDownloadTracker/);
  assert.match(downloadTracker, /from\("downloads"\)\.insert/);
  assert.match(downloadTracker, /platform: event\.platform/);
  assert.match(downloadTracker, /version: event\.releaseTag/);
  assert.doesNotMatch(downloadTracker, /created_at/);
  assert.match(routeHandler, /recordDownload/);
  assert.match(routeHandler, /status: 302/);
  assert.match(routeHandler, /resolveLatestReleaseAsset/);
  assert.match(macRoute, /handleDownloadRequest\("mac"\)/);
  assert.match(windowsRoute, /handleDownloadRequest\("windows"\)/);
  assert.match(linuxRoute, /handleDownloadRequest\("linux"\)/);
});
