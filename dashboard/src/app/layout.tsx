import type { Metadata } from 'next';
import { GlassNav } from '@/components/GlassNav';
import './globals.css';

export const metadata: Metadata = {
  title: 'PushOver Dashboard',
  description: 'PushOver Serverless Platform',
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="ko" className="antialiased">
      <body className="min-h-full flex flex-col">
        <GlassNav />
        <main className="pt-12">{children}</main>
      </body>
    </html>
  );
}
