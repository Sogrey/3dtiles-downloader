use anyhow::{Context, Result};
use std::collections::HashSet;
use std::sync::Arc;
use tiles_downloader::cli::Args;
use tiles_downloader::downloader::{DownloadQueue, DownloadTask, Downloader, HttpClient};
use tiles_downloader::parser::Tileset;
use tiles_downloader::utils::{ensure_dir, extract_relative_path, extract_filename};
use log::info;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // 解析命令行参数
    let args = Args::parse_args();

    info!("开始下载 3D Tiles 数据");
    info!("URL: {}", args.url);
    info!("输出目录: {}", args.dir);
    info!("线程数: {}", args.threads);

    // 确保输出目录存在
    ensure_dir(&args.dir)
        .await
        .with_context(|| format!("创建目录失败: {}", args.dir))?;

    // 创建 HTTP 客户端（设置更长的超时时间）
    let http_client = Arc::new(
        HttpClient::new(args.referer.clone(), 300, args.insecure)
            .context("创建 HTTP 客户端失败")?,
    );

    if args.insecure {
        info!("警告: 已启用不安全模式，跳过 SSL 证书验证");
    }

    // 用于存储所有需要下载的资源文件（不包括 tileset.json）
    let resource_urls = Arc::new(Mutex::new(Vec::new()));
    // 用于记录已处理的 tileset URL，避免重复处理
    let processed_tilesets = Arc::new(Mutex::new(HashSet::new()));

    // 递归处理 tileset
    process_tileset(
        &args.url,
        &args.dir,
        http_client.clone(),
        resource_urls.clone(),
        processed_tilesets.clone(),
    )
    .await?;

    // 获取所有资源 URL
    let resource_urls = resource_urls.lock().await;
    let total_resources = resource_urls.len();
    info!("共找到 {} 个资源文件", total_resources);

    if total_resources == 0 {
        info!("没有需要下载的资源文件");
        return Ok(());
    }

    // 分段处理
    let start = args.start;
    let end = if args.end == 0 || args.end > total_resources {
        total_resources
    } else {
        args.end
    };

    let uris_to_download: Vec<_> = resource_urls
        .iter()
        .skip(start)
        .take(end - start)
        .cloned()
        .collect();
    
    drop(resource_urls); // 释放锁
    
    info!("本次下载: {} 个文件 (位置: {} - {})", uris_to_download.len(), start, end);

    if uris_to_download.is_empty() {
        info!("没有需要下载的文件");
        return Ok(());
    }

    // 构建下载任务
    let tasks: Vec<DownloadTask> = uris_to_download
        .iter()
        .map(|url| {
            let relative_path = extract_relative_path(&args.url, url);
            log::debug!("URL: {} -> 相对路径: {}", url, relative_path);
            DownloadTask {
                url: url.clone(),
                relative_path,
            }
        })
        .collect();

    // 打印前 10 个任务（避免输出过多）
    for (i, task) in tasks.iter().take(10).enumerate() {
        info!("任务 {}: {} -> {}", i + 1, task.url, task.relative_path);
    }
    if tasks.len() > 10 {
        info!("... 还有 {} 个任务", tasks.len() - 10);
    }

    // 创建下载队列
    let queue = DownloadQueue::new(tasks);

    // 创建下载器并启动
    let downloader = Downloader::new(
        http_client,
        queue,
        args.dir.clone(),
        args.threads,
        uris_to_download.len() as u64,
    );

    downloader.run().await?;

    info!("下载完成!");

    Ok(())
}

/// 递归处理 tileset.json
async fn process_tileset(
    tileset_url: &str,
    output_dir: &str,
    http_client: Arc<HttpClient>,
    resource_urls: Arc<Mutex<Vec<String>>>,
    processed_tilesets: Arc<Mutex<HashSet<String>>>,
) -> Result<()> {
    // 检查是否已处理过
    {
        let mut processed = processed_tilesets.lock().await;
        if processed.contains(tileset_url) {
            return Ok(());
        }
        processed.insert(tileset_url.to_string());
    }

    info!("正在处理: {}", tileset_url);

    // 下载 tileset.json
    let tileset_json = http_client
        .get_text(tileset_url)
        .await
        .with_context(|| format!("下载 tileset 失败: {}", tileset_url))?;

    // 解析 tileset
    let tileset = Tileset::from_json(&tileset_json)
        .with_context(|| format!("解析 tileset 失败: {}", tileset_url))?;

    // 保存 tileset.json 文件
    let tileset_filename = extract_filename(tileset_url);
    let tileset_path = std::path::Path::new(output_dir).join(&tileset_filename);
    
    // 创建必要的父目录
    if let Some(parent) = tileset_path.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }
    
    tokio::fs::write(&tileset_path, &tileset_json)
        .await
        .with_context(|| format!("保存 {} 失败", tileset_filename))?;
    
    info!("✓ 已保存: {}", tileset_filename);

    // 获取所有资源 URI
    let uris = tileset.get_all_uris(tileset_url);

    // 分类处理：子 tileset 和资源文件
    for uri in uris {
        if Tileset::is_tileset_json(&uri) {
            // 递归处理子 tileset
            Box::pin(process_tileset(
                &uri,
                output_dir,
                http_client.clone(),
                resource_urls.clone(),
                processed_tilesets.clone(),
            ))
            .await?;
        } else {
            // 添加到资源列表
            let mut resources = resource_urls.lock().await;
            if !resources.contains(&uri) {
                resources.push(uri);
            }
        }
    }

    Ok(())
}
