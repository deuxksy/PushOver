use worker::*;

/// KV 저장소 핸들러
pub struct Kv {
    kv: KvStore,
}

impl Kv {
    /// Env에서 KVStore 생성
    pub fn new(env: &Env) -> Result<Self> {
        let kv = env.kv("PUSHOVER_CACHE")?;
        Ok(Self { kv })
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
