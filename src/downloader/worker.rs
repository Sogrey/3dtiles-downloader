use crate::downloader::{DownloadQueue, DownloadTask, HttpClient};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::task::JoinSet;

/// 下载工作器
pub struct Downloader {
    /// HTTP 客户端
    http_client: Arc<HttpClient>,
    /// 下载队列
    queue: DownloadQueue,
    /// 输出目录
    output_dir: String,
    /// 线程数
    threads: usize,
    /// 进度条
    progress: ProgressBar,
}

impl Downloader {
    /// 创建新的下载器
    pub fn new(
        http_client: Arc<HttpClient>,
        queue: DownloadQueue,
        output_dir: String,
        threads: usize,
        total: u64,
    ) -> Self {
        let progress = ProgressBar::new(total);
        progress.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("=>-"),
        );

        Self {
            http_client,
            queue,
            output_dir,
            threads,
            progress,
        }
    }

    /// 启动下载
    pub async fn run(&self) -> Result<()> {
        let mut join_set = JoinSet::new();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.threads));

        // 获取所有任务
        let mut tasks = Vec::new();
        while let Some(task) = self.queue.pop().await {
            tasks.push(task);
        }

        self.progress.println(format!("开始下载 {} 个文件...", tasks.len()));

        // 启动下载任务
        for task in tasks {
            let permit = semaphore.clone().acquire_owned().await?;
            let http_client = self.http_client.clone();
            let output_dir = self.output_dir.clone();
            let progress = self.progress.clone();

            join_set.spawn(async move {
                let _permit = permit;
                
                match download_file(&http_client, &task, &output_dir).await {
                    Ok(size) => {
                        progress.inc(1);
                        progress.println(format!("✅ {} ({:.2} KB)", task.relative_path, size as f64 / 1024.0));
                    }
                    Err(e) => {
                        progress.println(format!("❌ {} - {}", task.relative_path, e));
                    }
                }
            });
        }

        // 等待所有任务完成
        while join_set.join_next().await.is_some() {}

        self.progress.finish_with_message("下载完成");
        Ok(())
    }
}

/// 下载单个文件，返回文件大小
async fn download_file(http_client: &HttpClient, task: &DownloadTask, output_dir: &str) -> Result<usize> {
    // 构建输出路径
    let output_path = Path::new(output_dir).join(&task.relative_path);

    // 创建父目录
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await?;
        }
    }

    // 检查文件是否已存在
    if output_path.exists() {
        let metadata = std::fs::metadata(&output_path)?;
        return Ok(metadata.len() as usize);
    }

    // 下载数据
    let data = http_client.get_bytes(&task.url).await?;

    // 保存文件
    fs::write(&output_path, &data).await?;

    Ok(data.len())
}
