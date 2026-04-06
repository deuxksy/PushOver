'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { useSettings } from './settings/hooks/useSettings';
import { pushOverAPI } from '@/lib/api';

export default function Home() {
  const [showModal, setShowModal] = useState(false);
  const [message, setMessage] = useState('');
  const [title, setTitle] = useState('');
  const [sending, setSending] = useState(false);
  const [showBanner, setShowBanner] = useState(false);
  const [imageBase64, setImageBase64] = useState<string | null>(null);
  const [imagePreview, setImagePreview] = useState<string | null>(null);
  const { settings, isLoading, hasRequiredSettings } = useSettings();

  useEffect(() => {
    setShowBanner(!hasRequiredSettings);
  }, [hasRequiredSettings]);

  const handleSend = async () => {
    setSending(true);
    try {
      await pushOverAPI.sendMessage({ message, title: title || undefined, image: imageBase64 || undefined });
      alert('메시지 전송 성공!');
      setShowModal(false);
      setMessage('');
      setTitle('');
      setImageBase64(null);
      setImagePreview(null);
    } catch (error) {
      alert('전송 실패: ' + (error instanceof Error ? error.message : error));
    } finally {
      setSending(false);
    }
  };

  const handleImageChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (ev) => {
      setImagePreview(ev.target?.result as string);
    };
    reader.readAsDataURL(file);

    const b64Reader = new FileReader();
    b64Reader.onload = (ev) => {
      const result = ev.target?.result as string;
      const base64 = result.split(',')[1];
      setImageBase64(base64);
    };
    b64Reader.readAsDataURL(file);
  };

  const clearImage = () => {
    setImageBase64(null);
    setImagePreview(null);
  };

  const closeModal = () => {
    setShowModal(false);
    setMessage('');
    setTitle('');
    clearImage();
  };

  // Modal scroll lock (Spec 7: 접근성 — 모달 오픈 시 overflow: hidden)
  useEffect(() => {
    if (showModal) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => { document.body.style.overflow = ''; };
  }, [showModal]);

  if (isLoading) {
    return (
      <div className="min-h-screen bg-black p-8">
        <div className="animate-pulse space-y-4 max-w-[980px] mx-auto">
          <div className="h-16 bg-white/10 rounded-2xl" />
          <div className="h-32 bg-white/10 rounded-2xl" />
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen">
      {/* Section 1: Black Hero */}
      <section className="bg-black text-white text-center py-20 px-6">
        <div className="max-w-[980px] mx-auto">
          <h1
            className="font-semibold mb-3"
            style={{ fontSize: 'clamp(28px, 5vw, 56px)', lineHeight: 1.07, letterSpacing: '-0.28px' }}
          >
            PushOver Serverless Platform
          </h1>
          <p className="text-white/60 mb-8" style={{ fontSize: '17px' }}>
            메시지 전송, 웹훅 관리, 기록 조회
          </p>

          {showBanner && (
            <div className="bg-amber-900/30 border border-amber-700 rounded-2xl p-4 mb-6 max-w-md mx-auto text-left">
              <div className="flex items-center gap-3">
                <span className="text-xl">⚠️</span>
                <div className="flex-1">
                  <p className="font-medium text-amber-200 text-sm">PushOver 설정이 필요합니다</p>
                  <p className="text-xs text-white/50">Settings에서 API Token과 User Key를 설정하세요</p>
                </div>
                <Link
                  href="/settings"
                  className="px-4 py-2 bg-amber-600 text-white rounded-[980px] text-sm font-medium hover:bg-amber-700 transition-colors whitespace-nowrap"
                >
                  설정하기
                </Link>
              </div>
            </div>
          )}

          <button
            onClick={() => setShowModal(true)}
            className="px-8 py-3 bg-[var(--color-apple-blue)] text-white rounded-[980px] font-medium hover:bg-[var(--color-apple-blue-hover)] transition-colors"
            style={{ fontSize: '17px' }}
          >
            메시지 보내기
          </button>
        </div>
      </section>

      {/* Section 2: Light Content */}
      <section className="bg-[var(--color-apple-light)] py-16 px-6">
        <div className="max-w-[980px] mx-auto grid grid-cols-1 sm:grid-cols-2 gap-6">
          <div className="bg-white rounded-[16px] p-6" style={{ boxShadow: 'var(--shadow-apple)' }}>
            <p className="text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-2">Platform</p>
            <p className="text-[21px] font-bold text-[var(--color-apple-near-black)]">Cloudflare Workers</p>
            <p className="text-sm text-zinc-500 mt-1">Serverless 메시지 전송</p>
          </div>
          <div className="bg-white rounded-[16px] p-6" style={{ boxShadow: 'var(--shadow-apple)' }}>
            <p className="text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-2">Storage</p>
            <p className="text-[21px] font-bold text-[var(--color-apple-near-black)]">D1 + KV + R2</p>
            <p className="text-sm text-zinc-500 mt-1">메시지 기록, 설정, 이미지</p>
          </div>
        </div>
      </section>

      {/* Message Modal */}
      {showModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center" onClick={closeModal}>
          <div className="absolute inset-0 bg-black/50" />
          <div
            className="relative bg-white rounded-[16px] max-w-md w-full mx-4 p-6"
            style={{ boxShadow: 'var(--shadow-apple)' }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 className="text-lg font-semibold mb-4 text-[var(--color-apple-near-black)]">메시지 보내기</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-1.5">
                  제목 (선택)
                </label>
                <input
                  type="text"
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  placeholder="메시지 제목"
                  className="w-full px-3 py-3 rounded-[12px] bg-[var(--color-apple-light)] border-0 text-[var(--color-apple-near-black)] focus:outline-none focus:ring-2 focus:ring-[var(--color-apple-blue)]"
                />
              </div>
              <div>
                <label className="block text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-1.5">
                  메시지
                </label>
                <textarea
                  value={message}
                  onChange={(e) => setMessage(e.target.value)}
                  placeholder="전송할 메시지"
                  rows={4}
                  required
                  className="w-full px-3 py-3 rounded-[12px] bg-[var(--color-apple-light)] border-0 text-[var(--color-apple-near-black)] focus:outline-none focus:ring-2 focus:ring-[var(--color-apple-blue)]"
                />
              </div>
              <div>
                <label className="block text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-1.5">
                  이미지 첨부 (선택)
                </label>
                {imagePreview ? (
                  <div className="relative">
                    <img src={imagePreview} alt="preview" className="max-h-40 rounded-[12px]" />
                    <button
                      type="button"
                      onClick={clearImage}
                      className="absolute top-1 right-1 w-6 h-6 bg-black/70 text-white rounded-full text-xs flex items-center justify-center hover:bg-[var(--color-apple-error)]"
                    >
                      ✕
                    </button>
                  </div>
                ) : (
                  <input
                    type="file"
                    accept="image/*"
                    onChange={handleImageChange}
                    className="w-full text-sm text-zinc-400 file:mr-4 file:py-2 file:px-4 file:rounded-[980px] file:border file:border-[var(--color-apple-blue)] file:text-sm file:font-medium file:bg-transparent file:text-[var(--color-apple-blue)] hover:file:bg-[var(--color-apple-light)]"
                  />
                )}
              </div>
              <div className="flex gap-3 justify-end pt-2">
                <button
                  onClick={closeModal}
                  className="px-5 py-2.5 border border-[var(--color-apple-blue)] rounded-[980px] font-medium text-[var(--color-apple-blue)] hover:bg-[var(--color-apple-light)] transition-colors"
                >
                  취소
                </button>
                <button
                  onClick={handleSend}
                  disabled={!message || sending}
                  className="px-5 py-2.5 bg-[var(--color-apple-blue)] text-white rounded-[980px] font-medium hover:bg-[var(--color-apple-blue-hover)] disabled:opacity-50 transition-colors"
                >
                  {sending ? '전송 중...' : '전송'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
