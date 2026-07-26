import type { Metadata } from "next";
import { Analytics } from "@vercel/analytics/next";
import "@fontsource-variable/manrope";
import "./globals.css";

export const metadata: Metadata = {
  title: "Ducky — Your Desktop. Your Buddy.",
  description:
    "A playful, open-source AI desktop companion for chats, focus sessions, sticky notes, reminders, and more.",
  keywords: [
    "Ducky",
    "desktop companion",
    "AI assistant",
    "Pomodoro",
    "reminders",
    "open source",
  ],
  openGraph: {
    title: "Ducky — Your Desktop. Your Buddy.",
    description:
      "Meet the playful AI companion that lives on your desktop and helps make work a little more fun.",
    type: "website",
  },
  twitter: {
    card: "summary",
    title: "Ducky — Your Desktop. Your Buddy.",
    description:
      "A playful, open-source AI desktop companion for chat, focus, notes, and reminders.",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>
        {children}
        <Analytics />
      </body>
    </html>
  );
}
