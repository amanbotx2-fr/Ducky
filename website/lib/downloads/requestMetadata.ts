export type DownloadRequestMetadata = {
  browser: string | null;
  operatingSystem: string | null;
  referrer: string | null;
  country: string | null;
};

function inferBrowser(userAgent: string): string | null {
  if (/\bEdg(?:A|iOS)?\//.test(userAgent)) {
    return "Edge";
  }

  if (/\b(?:OPR|Opera)\//.test(userAgent)) {
    return "Opera";
  }

  if (/\bSamsungBrowser\//.test(userAgent)) {
    return "Samsung Internet";
  }

  if (/\b(?:Firefox|FxiOS)\//.test(userAgent)) {
    return "Firefox";
  }

  if (/\b(?:Chrome|CriOS)\//.test(userAgent)) {
    return "Chrome";
  }

  if (/\bVersion\/[\d.]+.*\bSafari\//.test(userAgent)) {
    return "Safari";
  }

  return null;
}

function inferOperatingSystem(userAgent: string): string | null {
  if (/\b(?:iPhone|iPad|iPod)\b/.test(userAgent)) {
    return "iOS";
  }

  if (/\bAndroid\b/.test(userAgent)) {
    return "Android";
  }

  if (/\bCrOS\b/.test(userAgent)) {
    return "Chrome OS";
  }

  if (/\bWindows(?: Phone)?\b/.test(userAgent)) {
    return "Windows";
  }

  if (/\b(?:Macintosh|Mac OS X)\b/.test(userAgent)) {
    return "macOS";
  }

  if (/\bLinux\b/.test(userAgent)) {
    return "Linux";
  }

  return null;
}

function normalizeReferrer(value: string | null): string | null {
  if (!value) {
    return null;
  }

  try {
    const referrer = new URL(value);
    if (referrer.protocol !== "http:" && referrer.protocol !== "https:") {
      return null;
    }

    const hostname = referrer.hostname.toLowerCase().replace(/^www\./, "");
    return hostname || null;
  } catch {
    return null;
  }
}

function normalizeCountry(value: string | null): string | null {
  const country = value?.trim().toUpperCase();
  return country && /^[A-Z]{2}$/.test(country) ? country : null;
}

export function getDownloadRequestMetadata(
  request: Request,
): DownloadRequestMetadata {
  const userAgent = request.headers.get("user-agent") ?? "";

  return {
    browser: inferBrowser(userAgent),
    operatingSystem: inferOperatingSystem(userAgent),
    referrer: normalizeReferrer(request.headers.get("referer")),
    country: normalizeCountry(
      request.headers.get("x-vercel-ip-country"),
    ),
  };
}
