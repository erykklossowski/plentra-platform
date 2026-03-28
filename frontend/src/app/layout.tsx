import type { Metadata } from "next";
import { Manrope, Inter } from "next/font/google";
import TopNav from "@/components/layout/TopNav";
import SideNav from "@/components/layout/SideNav";
import MobileNav from "@/components/layout/MobileNav";
import "./globals.css";

const manrope = Manrope({
  variable: "--font-manrope",
  subsets: ["latin"],
});

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Plentra Research Intelligence Platform",
  description: "B2B energy market analytics for Polish wholesale electricity market",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={`dark ${manrope.variable} ${inter.variable}`}>
      <head>
        <link
          href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap"
          rel="stylesheet"
        />
      </head>
      <body className="bg-background text-on-surface font-body min-h-screen">
        <TopNav />
        <SideNav />
        <MobileNav />
        <main className="ml-0 md:ml-64 pt-16">{children}</main>
      </body>
    </html>
  );
}
