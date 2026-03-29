use worker::*;
use wasm_bindgen::JsValue;
use crate::types::{DbMessage, DbWebhook, DbFailedDelivery, DbApiToken};

pub struct Db {
    d1: D1Database,
}

impl Db {
    pub fn new(ctx: &RouteContext<()>) -> Result<Self> {
        let d1 = ctx.env.d1("DB")?;
        Ok(Self { d1 })
    }

    pub fn from_env(env: &Env) -> Result<Self> {
        let d1 = env.d1("DB")?;
        Ok(Self { d1 })
    }

    // ---- Messages ----

    pub async fn insert_message(
        &self,
        id: &str,
        user_key: &str,
        message: &str,
        title: Option<&str>,
        priority: i32,
        sound: Option<&str>,
        device: Option<&str>,
        url: Option<&str>,
        url_title: Option<&str>,
        html: bool,
        status: &str,
        receipt: Option<&str>,
        api_token: Option<&str>,
    ) -> Result<()> {
        self.d1
            .prepare(
                "INSERT INTO messages (id, user_key, message, title, priority, sound, device, url, url_title, html, status, receipt, api_token, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"
            )
            .bind(&[
                JsValue::from_str(id),
                JsValue::from_str(user_key),
                JsValue::from_str(message),
                title.map(JsValue::from_str).unwrap_or(JsValue::NULL),
                JsValue::from_f64(priority as f64),
                sound.map(JsValue::from_str).unwrap_or(JsValue::NULL),
                device.map(JsValue::from_str).unwrap_or(JsValue::NULL),
                url.map(JsValue::from_str).unwrap_or(JsValue::NULL),
                url_title.map(JsValue::from_str).unwrap_or(JsValue::NULL),
                JsValue::from_f64(if html { 1.0 } else { 0.0 }),
                JsValue::from_str(status),
                receipt.map(JsValue::from_str).unwrap_or(JsValue::NULL),
                api_token.map(JsValue::from_str).unwrap_or(JsValue::NULL),
            ])?
            .run()
            .await?;
        Ok(())
    }

    pub async fn get_message_by_id(&self, id: &str) -> Result<Option<DbMessage>> {
        self.d1
            .prepare("SELECT * FROM messages WHERE id = ?")
            .bind(&[JsValue::from_str(id)])?
            .first(None)
            .await
    }

    pub async fn get_message_by_receipt(&self, receipt: &str) -> Result<Option<DbMessage>> {
        self.d1
            .prepare("SELECT * FROM messages WHERE receipt = ?")
            .bind(&[JsValue::from_str(receipt)])?
            .first(None)
            .await
    }

    pub async fn get_messages_by_user(&self, user_key: &str, limit: u32) -> Result<Vec<DbMessage>> {
        let result = self.d1
            .prepare("SELECT * FROM messages WHERE user_key = ? ORDER BY created_at DESC LIMIT ?")
            .bind(&[JsValue::from_str(user_key), JsValue::from_f64(limit as f64)])?
            .all()
            .await?;
        result.results::<DbMessage>()
    }

    pub async fn update_message_status(&self, id: &str, status: &str) -> Result<()> {
        self.d1
            .prepare("UPDATE messages SET status = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(&[JsValue::from_str(status), JsValue::from_str(id)])?
            .run()
            .await?;
        Ok(())
    }

    pub async fn update_message_receipt(&self, id: &str, receipt: &str) -> Result<()> {
        self.d1
            .prepare("UPDATE messages SET receipt = ?, status = 'sent', sent_at = datetime('now'), updated_at = datetime('now') WHERE id = ?")
            .bind(&[JsValue::from_str(receipt), JsValue::from_str(id)])?
            .run()
            .await?;
        Ok(())
    }

    pub async fn export_all_messages(&self) -> Result<Vec<DbMessage>> {
        let result = self.d1
            .prepare("SELECT * FROM messages ORDER BY created_at DESC LIMIT 10000")
            .bind(&[])?
            .all()
            .await?;
        result.results::<DbMessage>()
    }

    pub async fn update_message_image_url(&self, id: &str, image_url: &str) -> Result<()> {
        self.d1.prepare("UPDATE messages SET image_url = ? WHERE id = ?")
            .bind(&[JsValue::from_str(image_url), JsValue::from_str(id)])?
            .run()
            .await?;
        Ok(())
    }

    pub async fn acknowledge_message(&self, id: &str) -> Result<()> {
        self.d1
            .prepare("UPDATE messages SET status = 'acknowledged', acknowledged_at = datetime('now'), updated_at = datetime('now') WHERE id = ?")
            .bind(&[JsValue::from_str(id)])?
            .run()
            .await?;
        Ok(())
    }

    // ---- Webhooks ----

    pub async fn insert_webhook(
        &self,
        id: &str,
        user_key: &str,
        url: &str,
        secret: &str,
        events: &str,
    ) -> Result<()> {
        self.d1
            .prepare(
                "INSERT INTO webhooks (id, user_key, url, secret, events, active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 1, datetime('now'), datetime('now'))"
            )
            .bind(&[
                JsValue::from_str(id),
                JsValue::from_str(user_key),
                JsValue::from_str(url),
                JsValue::from_str(secret),
                JsValue::from_str(events),
            ])?
            .run()
            .await?;
        Ok(())
    }

    pub async fn get_webhooks_by_user(&self, user_key: &str) -> Result<Vec<DbWebhook>> {
        let result = self.d1
            .prepare("SELECT * FROM webhooks WHERE user_key = ? AND active = 1")
            .bind(&[JsValue::from_str(user_key)])?
            .all()
            .await?;
        result.results::<DbWebhook>()
    }

    pub async fn get_webhook_by_id(&self, id: &str) -> Result<Option<DbWebhook>> {
        self.d1
            .prepare("SELECT * FROM webhooks WHERE id = ?")
            .bind(&[JsValue::from_str(id)])?
            .first(None)
            .await
    }

    pub async fn delete_webhook(&self, id: &str, user_key: &str) -> Result<bool> {
        let result = self.d1
            .prepare("DELETE FROM webhooks WHERE id = ? AND user_key = ?")
            .bind(&[JsValue::from_str(id), JsValue::from_str(user_key)])?
            .run()
            .await?;
        let meta = result.meta()?.unwrap();
        Ok(meta.changes.unwrap_or(0) > 0)
    }

    pub async fn update_webhook_last_triggered(&self, id: &str) -> Result<()> {
        self.d1
            .prepare("UPDATE webhooks SET last_triggered_at = datetime('now'), updated_at = datetime('now') WHERE id = ?")
            .bind(&[JsValue::from_str(id)])?
            .run()
            .await?;
        Ok(())
    }

    // ---- Webhook Deliveries ----

    pub async fn insert_webhook_delivery(
        &self,
        id: &str,
        webhook_id: &str,
        message_id: &str,
        event_type: &str,
    ) -> Result<()> {
        self.d1
            .prepare(
                "INSERT INTO webhook_deliveries (id, webhook_id, message_id, event_type, status, retry_count, created_at, updated_at) VALUES (?, ?, ?, ?, 'pending', 0, datetime('now'), datetime('now'))"
            )
            .bind(&[
                JsValue::from_str(id),
                JsValue::from_str(webhook_id),
                JsValue::from_str(message_id),
                JsValue::from_str(event_type),
            ])?
            .run()
            .await?;
        Ok(())
    }

    pub async fn update_webhook_delivery_status(
        &self,
        id: &str,
        status: &str,
        response_code: Option<i32>,
        response_body: Option<&str>,
    ) -> Result<()> {
        self.d1
            .prepare("UPDATE webhook_deliveries SET status = ?, response_code = ?, response_body = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(&[
                JsValue::from_str(status),
                response_code.map(|c| JsValue::from_f64(c as f64)).unwrap_or(JsValue::NULL),
                response_body.map(JsValue::from_str).unwrap_or(JsValue::NULL),
                JsValue::from_str(id),
            ])?
            .run()
            .await?;
        Ok(())
    }

    // ---- Failed Deliveries ----

    pub async fn get_failed_deliveries(&self, max_attempts: u32) -> Result<Vec<DbFailedDelivery>> {
        let result = self.d1
            .prepare("SELECT * FROM failed_deliveries WHERE attempt_count < ?")
            .bind(&[JsValue::from_f64(max_attempts as f64)])?
            .all()
            .await?;
        result.results::<DbFailedDelivery>()
    }

    pub async fn upsert_failed_delivery(&self, message_id: &str, error: &str) -> Result<()> {
        let existing: Option<DbFailedDelivery> = self.d1
            .prepare("SELECT * FROM failed_deliveries WHERE message_id = ?")
            .bind(&[JsValue::from_str(message_id)])?
            .first(None)
            .await?;

        if existing.is_some() {
            self.d1
                .prepare("UPDATE failed_deliveries SET attempt_count = attempt_count + 1, last_attempt_at = datetime('now'), error_message = ?, updated_at = datetime('now') WHERE message_id = ?")
                .bind(&[JsValue::from_str(error), JsValue::from_str(message_id)])?
                .run()
                .await?;
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            self.d1
                .prepare(
                    "INSERT INTO failed_deliveries (id, message_id, attempt_count, last_attempt_at, error_message, created_at, updated_at) VALUES (?, ?, 1, datetime('now'), ?, datetime('now'), datetime('now'))"
                )
                .bind(&[
                    JsValue::from_str(&id),
                    JsValue::from_str(message_id),
                    JsValue::from_str(error),
                ])?
                .run()
                .await?;
        }
        Ok(())
    }

    pub async fn delete_failed_delivery(&self, message_id: &str) -> Result<()> {
        self.d1
            .prepare("DELETE FROM failed_deliveries WHERE message_id = ?")
            .bind(&[JsValue::from_str(message_id)])?
            .run()
            .await?;
        Ok(())
    }

    // ---- API Tokens ----

    /// Bearer 토큰 검증: 활성 토큰이면 user_key 반환
    pub async fn validate_token(&self, token: &str) -> Result<Option<String>> {
        let result: Option<DbApiToken> = self.d1
            .prepare("SELECT * FROM api_tokens WHERE token = ? AND active = 1")
            .bind(&[JsValue::from_str(token)])?
            .first(None)
            .await?;

        if result.is_some() {
            // last_used_at 업데이트 (non-blocking)
            let _ = self.d1
                .prepare("UPDATE api_tokens SET last_used_at = datetime('now'), updated_at = datetime('now') WHERE token = ?")
                .bind(&[JsValue::from_str(token)])?
                .run()
                .await;
        }

        Ok(result.map(|t| t.user_key))
    }

    /// 새 API 토큰 등록
    pub async fn register_token(&self, token: &str, user_key: &str, name: Option<&str>) -> Result<()> {
        self.d1
            .prepare("INSERT INTO api_tokens (token, user_key, name, active, created_at, updated_at) VALUES (?, ?, ?, 1, datetime('now'), datetime('now'))")
            .bind(&[
                JsValue::from_str(token),
                JsValue::from_str(user_key),
                name.map(JsValue::from_str).unwrap_or(JsValue::NULL),
            ])?
            .run()
            .await?;
        Ok(())
    }

    /// 토큰 비활성화
    pub async fn deactivate_token(&self, token: &str) -> Result<bool> {
        let result = self.d1
            .prepare("UPDATE api_tokens SET active = 0, updated_at = datetime('now') WHERE token = ?")
            .bind(&[JsValue::from_str(token)])?
            .run()
            .await?;
        let meta = result.meta()?.unwrap();
        Ok(meta.changes.unwrap_or(0) > 0)
    }
}
