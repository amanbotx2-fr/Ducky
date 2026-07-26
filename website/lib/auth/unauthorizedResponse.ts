import "server-only";

const unauthorizedHtml = `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="robots" content="noindex, nofollow">
    <title>403 Unauthorized — Ducky</title>
    <style>
      :root { color-scheme: light; font-family: Manrope, ui-rounded, system-ui, sans-serif; }
      * { box-sizing: border-box; }
      body {
        min-width: 320px;
        min-height: 100vh;
        margin: 0;
        display: grid;
        place-items: center;
        padding: 20px;
        background: #fff9ef;
        color: #111111;
      }
      main {
        width: min(100%, 560px);
        padding: 32px;
        border: 3px solid #111111;
        border-radius: 22px;
        background: #fff9ef;
        box-shadow: 8px 8px 0 #111111;
      }
      .mark {
        display: inline-grid;
        width: 56px;
        height: 56px;
        place-items: center;
        border: 2px solid #111111;
        border-radius: 15px;
        background: #ff91aa;
        box-shadow: 3px 3px 0 #111111;
        font-size: 24px;
        font-weight: 900;
      }
      .eyebrow {
        margin: 28px 0 0;
        color: #ff6a3d;
        font-size: 12px;
        font-weight: 900;
        letter-spacing: .13em;
        text-transform: uppercase;
      }
      h1 {
        margin: 8px 0 0;
        font-size: clamp(32px, 8vw, 42px);
        line-height: 1;
        letter-spacing: -.05em;
      }
      p {
        max-width: 470px;
        margin: 18px 0 0;
        color: rgba(17, 17, 17, .68);
        font-size: 16px;
        font-weight: 650;
        line-height: 1.7;
      }
      nav {
        display: flex;
        flex-wrap: wrap;
        gap: 12px;
        margin-top: 30px;
      }
      a {
        min-height: 44px;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 10px 16px;
        border: 2px solid #111111;
        border-radius: 12px;
        background: #ffd54a;
        box-shadow: 3px 3px 0 #111111;
        color: #111111;
        font-size: 14px;
        font-weight: 900;
        text-decoration: none;
      }
      a.secondary { background: #fff9ef; }
      a:focus-visible { outline: 4px solid rgba(255, 106, 61, .35); outline-offset: 2px; }
      @media (max-width: 480px) {
        main { padding: 24px 20px; }
        nav, a { width: 100%; }
      }
    </style>
  </head>
  <body>
    <main>
      <span class="mark" aria-hidden="true">!</span>
      <p class="eyebrow">Error 403</p>
      <h1>Unauthorized</h1>
      <p>This GitHub account is signed in, but it is not approved to access Ducky's internal analytics.</p>
      <nav aria-label="Unauthorized actions">
        <a href="/api/auth/signout?callbackUrl=%2F">Sign out</a>
        <a class="secondary" href="/">Back to website</a>
      </nav>
    </main>
  </body>
</html>`;

export function createUnauthorizedResponse(): Response {
  return new Response(unauthorizedHtml, {
    status: 403,
    headers: {
      "Cache-Control": "no-store",
      "Content-Security-Policy":
        "default-src 'none'; style-src 'unsafe-inline'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'",
      "Content-Type": "text/html; charset=utf-8",
      "X-Content-Type-Options": "nosniff",
    },
  });
}
