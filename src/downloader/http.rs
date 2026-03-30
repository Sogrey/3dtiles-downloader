use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

/// HTTP 客户端封装
pub struct HttpClient {
    client: Client,
    referer: Option<String>,
    max_retries: u32,
}

impl HttpClient {
    /// 创建新的 HTTP 客户端
    pub fn new(referer: Option<String>, timeout_secs: u64, insecure: bool) -> Result<Self> {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .connect_timeout(Duration::from_secs(30))
            .tcp_keepalive(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .pool_idle_timeout(Duration::from_secs(60))
            .pool_max_idle_per_host(10);

        // 如果需要跳过 SSL 验证
        if insecure {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        let client = client_builder
            .build()
            .context("创建 HTTP 客户端失败")?;

        Ok(Self { 
            client, 
            referer,
            max_retries: 3,
        })
    }

    /// 获取文本内容
    pub async fn get_text(&self, url: &str) -> Result<String> {
        let mut last_error = None;
        
        for attempt in 1..=self.max_retries {
            match self.get_text_once(url).await {
                Ok(text) => return Ok(text),
                Err(e) => {
                    if attempt < self.max_retries {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                    last_error = Some(e);
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("未知错误")))
    }

    async fn get_text_once(&self, url: &str) -> Result<String> {
        let mut request = self.client.get(url);

        if let Some(ref referer) = self.referer {
            request = request.header("Referer", referer);
        }

        let response = request
            .send()
            .await
            .with_context(|| format!("请求失败: {}", url))?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("HTTP 错误: {} - {}", status, url);
        }

        let text = response
            .text()
            .await
            .with_context(|| format!("读取响应失败: {}", url))?;

        Ok(text)
    }

    /// 获取二进制内容（带重试）
    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let mut last_error = None;
        
        for attempt in 1..=self.max_retries {
            match self.get_bytes_once(url).await {
                Ok(bytes) => return Ok(bytes),
                Err(e) => {
                    eprintln!("下载失败 (尝试 {}/{}): {} - {}", attempt, self.max_retries, url, e);
                    if attempt < self.max_retries {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                    last_error = Some(e);
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("未知错误")))
    }

    async fn get_bytes_once(&self, url: &str) -> Result<Vec<u8>> {
        let mut request = self.client.get(url);

        if let Some(ref referer) = self.referer {
            request = request.header("Referer", referer);
        }

        let response = request
            .send()
            .await
            .with_context(|| format!("连接失败: {}", url))?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("HTTP 错误: {} - {}", status, url);
        }

        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("读取数据失败: {}", url))?;

        Ok(bytes.to_vec())
    }

    /// 获取客户端引用
    pub fn client(&self) -> &Client {
        &self.client
    }
}
