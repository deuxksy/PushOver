use worker::*;
use crate::db::Db;
use crate::types::{KV_PREFIX_TOKEN, TTL_TOKEN};

/// KV 저장소 핸들러
pub struct Kv {
    kv: KvStore,
}

impl Kv {
    /// Env에서 KVStore 생성
    pub fn new(env: &Env) -> Result<Self> {
        let kv = env.kv("KV")?;
        Ok(Self { kv })
    }

    /// 토큰 검증 (Cache-Aside 패턴)
    /// 1. KV 캐시 확인 → 2. Miss면 DB 조회 → 3. 캐시 갱신
    pub async fn validate_token(&self, db: &Db, token: &str) -> Result<Option<String>> {
        let cache_key = format!("{}{}", KV_PREFIX_TOKEN, token);

        // KV 캐시 확인
        if let Ok(Some(cached)) = self.kv.get(&cache_key).text().await {
            return Ok(Some(cached));
        }

        // Cache miss: DB 조회
        match db.validate_token(token).await? {
            Some(user_key) => {
                // 캐시에 저장 (TTL 1시간)
                let _ = self.kv.put(&cache_key, &user_key)?
                    .expiration_ttl(TTL_TOKEN)
                    .execute()
                    .await;
                Ok(Some(user_key))
            }
            None => Ok(None),
        }
    }

    /// 실패한 메시지를 KV에 백업
    pub async fn backup_failed_message(
        &self,
        id: &str,
        body: &str,
    ) -> Result<()> {
        let key = format!("failed:{}", id);
        self.kv.put(&key, body)?.execute().await?;
        Ok(())
    }

    /// KV에서 실패한 메시지 복원
    pub async fn get_failed_message(
        &self,
        id: &str,
    ) -> Result<Option<String>> {
        let key = format!("failed:{}", id);
        let value = self.kv.get(&key).text().await?;
        Ok(value)
    }

    /// KV에서 실패한 메시지 삭제
    pub async fn delete_failed_message(
        &self,
        id: &str,
    ) -> Result<()> {
        let key = format!("failed:{}", id);
        self.kv.delete(&key).await?;
        Ok(())
    }
}
