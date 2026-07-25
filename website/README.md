# Ducky Landing Website

The official Ducky landing page, built as a focused Next.js App Router site.
The current implementation includes the production hero from WEB-001, the
feature showcase from WEB-003, and the complete download and installation guide
from WEB-004. The one-page experience also includes the Support Ducky section
from WEB-009 and the production FAQ experience from WEB-010. The fixed
navigation connects Features, Download, Buy Me a Coffee, and FAQ with
active-section highlighting.

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
- `components/Support/` — support hero, support options, coffee mascot window,
  and shared capability strip
- `components/FAQ/` — FAQ hero, accessible accordions, help cards, and final CTA
- `components/Badge.tsx` — reusable neo-brutalist badge
- `components/BrandMark.tsx` — Ducky logo lockup
- `components/SectionContainer.tsx` — shared section width and gutters

## Brand assets

The official mascot is imported directly from `mascot pic/master.png`. The
coffee and FAQ poses in `assets/support/ducky-coffee.png` and
`assets/faq/ducky-faq.png` are identity-preserving variants derived from that
official art for their respective sections. The original visual briefs remain
in `refrence photos/`.

The current direct download URLs are centralized in `lib/releaseAssets.ts` and
match the verified v1.1.0 assets published by the release pipeline.

## Support destinations

Payment destinations are intentionally not guessed. Configure the real public
URLs through the deployment environment:

- `NEXT_PUBLIC_BUY_ME_A_COFFEE_URL`
- `NEXT_PUBLIC_GITHUB_SPONSORS_URL`
- `NEXT_PUBLIC_UPI_SUPPORT_URL`

These values are public links, not secrets. If a destination is omitted, its
card remains visible but the action is safely marked unavailable instead of
linking visitors to an unverified account.
