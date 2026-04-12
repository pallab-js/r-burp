import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "r-burp",
  description: "A lightweight, secure, privacy-oriented desktop proxy tool",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="h-full antialiased">
      <body className="min-h-full bg-cream text-cursor-dark flex flex-col">
        {children}
      </body>
    </html>
  );
}
