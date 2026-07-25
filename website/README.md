# Ducky Landing Website

The official Ducky landing page, built as a focused Next.js App Router site.
The current implementation includes the production hero from WEB-001, the
feature showcase from WEB-003, and the complete download and installation guide
from WEB-004. The fixed one-page navigation connects Features, Download,
Roadmap, and FAQ with active-section highlighting.

## Stack

- Next.js App Router with TypeScript
- Tailwind CSS
- Framer Motion
- Lucide Icons
- Locally bundled Manrope variable font

## Local development

Requires Node.js 22.13 or newer.

```bash
npm install
npm run dev
```

The local site is available at `http://localhost:3000`.

## Validation

```bash
npm run lint
npm run typecheck
npm test
```

`npm test` creates a production build and verifies the server-rendered hero,
feature showcase, download cards, and installation guide.

## Structure

- `app/` — page shell, metadata, and global design tokens
- `components/Navbar/` — responsive landing navigation
- `components/Hero/` — hero copy, downloads, mascot window, floating cards,
  and feature strip
- `components/Features/` — detailed feature cards, secondary mascot panel, and
  capability strip
- `components/Download/` — platform downloads, unsigned-app installation
  guidance, support links, and download benefits
- `components/ComingSoon/` — server-rendered Roadmap and FAQ placeholders
- `components/Badge.tsx` — reusable neo-brutalist badge
- `components/BrandMark.tsx` — Ducky logo lockup
- `components/SectionContainer.tsx` — shared section width and gutters

## Brand assets

The official mascot is imported directly from `mascot pic/master.png`. It is
not redrawn, generated, or replaced. The original visual brief remains in
`refrence photos/image.png`.

The current direct download URLs are centralized in `lib/releaseAssets.ts` and
match the verified v1.1.0 assets published by the release pipeline.
