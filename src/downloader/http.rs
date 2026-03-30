use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

/// HTTP 客户端封装
pub struct HttpClient {
    client: Client,
    referer: Option<String>,
}

impl HttpClient {
    /// 创建新的 HTTP 客户端
    pub fn new(referer: Option<String>, timeout_secs: u64) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .build()?;

        Ok(Self { client, referer })
    }

    /// 获取文本内容
    pub async fn get_text(&self, url: &str) -> Result<String> {
        let mut request = self.client.get(url);

        if let Some(ref referer) = self.referer {
            request = request.header("Referer", referer);
        }

        let response = request.send().await?;
        let text = response.text().await?;
        Ok(text)
    }

    /// 获取二进制内容
    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let mut request = self.client.get(url);

        if let Some(ref referer) = self.referer {
            request = request.header("Referer", referer);
        }

        let response = request.send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// 获取客户端引用
    pub fn client(&self) -> &Client {
        &self.client
    }
}
