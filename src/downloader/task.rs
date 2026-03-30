use std::sync::Arc;
use tokio::sync::Mutex;

/// 下载任务
#[derive(Debug, Clone)]
pub struct DownloadTask {
    /// URL
    pub url: String,
    /// 相对路径
    pub relative_path: String,
}

/// 下载队列
pub struct DownloadQueue {
    /// 任务列表
    tasks: Arc<Mutex<Vec<DownloadTask>>>,
}

impl DownloadQueue {
    /// 创建新的下载队列
    pub fn new(tasks: Vec<DownloadTask>) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(tasks)),
        }
    }

    /// 获取下一个任务
    pub async fn pop(&self) -> Option<DownloadTask> {
        let mut tasks = self.tasks.lock().await;
        tasks.pop()
    }

    /// 获取剩余任务数量
    pub async fn len(&self) -> usize {
        let tasks = self.tasks.lock().await;
        tasks.len()
    }

    /// 是否为空
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}
