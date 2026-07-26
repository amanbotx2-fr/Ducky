import { NextResponse } from "next/server";
import { auth } from "./auth";
import { isAllowedGitHubUsername } from "./lib/auth/authorization";
import { createUnauthorizedResponse } from "./lib/auth/unauthorizedResponse";

export const proxy = auth((request) => {
  if (!request.auth?.user) {
    const loginUrl = new URL("/login", request.nextUrl.origin);
    const callbackUrl = `${request.nextUrl.pathname}${request.nextUrl.search}`;

    loginUrl.searchParams.set("callbackUrl", callbackUrl);
    return NextResponse.redirect(loginUrl);
  }

  if (!isAllowedGitHubUsername(request.auth.user.githubUsername)) {
    return createUnauthorizedResponse();
  }

  return NextResponse.next();
});

export const config = {
  matcher: ["/admin/:path*"],
};
