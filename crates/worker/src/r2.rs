use std::collections::HashMap;
use worker::*;

pub struct R2 {
    bucket: Bucket,
}

impl R2 {
    pub fn new(env: &Env) -> Result<Self> {
        let bucket = env.bucket("R2")?;
        Ok(Self { bucket })
    }

    /// 이미지를 R2에 업로드
    pub async fn upload_image(
        &self,
        key: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<String> {
        let metadata = HashMap::from([
            ("content_type".to_string(), content_type.to_string()),
        ]);

        self.bucket.put(key, data.to_vec())
            .custom_metadata(metadata)
            .execute()
            .await?;

        Ok(format!("/images/{}", key))
    }

    /// 이미지 다운로드
    pub async fn get_image(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let object = self.bucket.get(key).execute().await?;

        match object {
            Some(obj) => {
                if let Some(body) = obj.body() {
                    let bytes = body.bytes().await?;
                    Ok(Some(bytes))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}
