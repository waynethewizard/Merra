import type { Metadata } from "next";
import { SiteFooter } from "@/components/SiteFooter";
import { SiteHeader } from "@/components/SiteHeader";
import "./globals.css";

export const metadata: Metadata = {
  title: {
    default: "Merra — A World That Remembers",
    template: "%s · Merra"
  },
  description:
    "Follow the making of an open-source historical simulation where worlds create causal histories and remember them imperfectly.",
  openGraph: {
    title: "Merra — A World That Remembers",
    description:
      "A living historical simulation built in public with Rust and Bevy.",
    type: "website"
  }
};

export default function RootLayout({
  children
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>
        <a className="skip-link" href="#main">
          Skip to content
        </a>
        <SiteHeader />
        <main id="main">{children}</main>
        <SiteFooter />
      </body>
    </html>
  );
}
