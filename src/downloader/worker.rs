use crate::downloader::{DownloadQueue, DownloadTask, HttpClient};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;

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
        http_client: HttpClient,
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
            http_client: Arc::new(http_client),
            queue,
            output_dir,
            threads,
            progress,
        }
    }

    /// 启动下载
    pub async fn run(&self) -> Result<()> {
        let semaphore = Arc::new(Semaphore::new(self.threads));
        let mut handles = vec![];

        loop {
            let permit = semaphore.clone().acquire_owned().await?;

            if let Some(task) = self.queue.pop().await {
                let http_client = self.http_client.clone();
                let output_dir = self.output_dir.clone();
                let progress = self.progress.clone();

                let handle = tokio::spawn(async move {
                    let _permit = permit;
                    if let Err(e) = download_file(&http_client, &task, &output_dir).await {
                        progress.println(format!("下载失败: {} - {}", task.url, e));
                    } else {
                        progress.inc(1);
                    }
                });

                handles.push(handle);
            } else {
                break;
            }
        }

        // 等待所有任务完成
        for handle in handles {
            handle.await?;
        }

        self.progress.finish_with_message("下载完成");
        Ok(())
    }
}

/// 下载单个文件
async fn download_file(http_client: &HttpClient, task: &DownloadTask, output_dir: &str) -> Result<()> {
    // 构建输出路径
    let output_path = Path::new(output_dir).join(&task.relative_path);

    // 创建父目录
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // 检查文件是否已存在
    if output_path.exists() {
        return Ok(());
    }

    // 下载数据
    let data = http_client.get_bytes(&task.url).await?;

    // 保存文件
    fs::write(&output_path, &data).await?;

    Ok(())
}
