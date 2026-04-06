'use client';

import { useState } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

const NAV_LINKS = [
  { href: '/', label: 'Home' },
  { href: '/history', label: 'History' },
  { href: '/settings', label: 'Settings' },
];

export function GlassNav() {
  const pathname = usePathname();
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <nav
      className="sticky top-0 z-50 h-12 flex items-center justify-between px-6 max-w-[980px] mx-auto"
      style={{
        background: 'rgba(0, 0, 0, 0.8)',
        backdropFilter: 'saturate(180%) blur(20px)',
        WebkitBackdropFilter: 'saturate(180%) blur(20px)',
      }}
      aria-label="Main navigation"
    >
      <Link href="/" className="text-white font-semibold" style={{ fontSize: '17px' }}>
        PushOver
      </Link>

      {/* Desktop links */}
      <div className="hidden sm:flex gap-5">
        {NAV_LINKS.map((link) => (
          <Link
            key={link.href}
            href={link.href}
            className="text-white/80 hover:text-white hover:underline"
            style={{ fontSize: '12px' }}
          >
            {link.label}
          </Link>
        ))}
      </div>

      {/* Mobile hamburger */}
      <button
        className="sm:hidden text-white text-lg"
        onClick={() => setMenuOpen(!menuOpen)}
        aria-label="Toggle menu"
      >
        {menuOpen ? '\u2715' : '\u2630'}
      </button>

      {/* Mobile overlay menu */}
      {menuOpen && (
        <div
          className="fixed inset-0 top-12 z-40 flex flex-col items-center gap-6 pt-12"
          style={{ background: 'rgba(0, 0, 0, 0.95)' }}
        >
          {NAV_LINKS.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              onClick={() => setMenuOpen(false)}
              className="text-white text-xl font-medium"
            >
              {link.label}
            </Link>
          ))}
        </div>
      )}
    </nav>
  );
}
