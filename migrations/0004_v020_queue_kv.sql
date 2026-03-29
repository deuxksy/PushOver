-- 0004: v0.2.0 Queue + KV + R2 스키마 변경

-- messages: image_url 컬럼 추가 (R2 이미지 참조)
ALTER TABLE messages ADD COLUMN image_url TEXT;
