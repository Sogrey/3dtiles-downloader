use anyhow::Result;
use tiles_downloader::cli::Args;
use tiles_downloader::downloader::{DownloadQueue, DownloadTask, Downloader, HttpClient};
use tiles_downloader::parser::Tileset;
use tiles_downloader::utils::{ensure_dir, extract_relative_path};
use log::info;

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
    ensure_dir(&args.dir).await?;

    // 创建 HTTP 客户端
    let http_client = HttpClient::new(args.referer.clone(), 300)?;

    // 下载并解析 tileset.json
    info!("正在解析 tileset.json...");
    let tileset_json = http_client.get_text(&args.url).await?;
    let tileset = Tileset::from_json(&tileset_json)?;

    // 获取所有资源 URI
    let uris = tileset.get_all_uris(&args.url);
    info!("共找到 {} 个资源文件", uris.len());

    // 分段处理
    let start = args.start;
    let end = if args.end == 0 || args.end > uris.len() {
        uris.len()
    } else {
        args.end
    };

    let uris_to_download: Vec<_> = uris.into_iter().skip(start).take(end - start).collect();
    info!("本次下载: {} 个文件 (位置: {} - {})", uris_to_download.len(), start, end);

    // 构建下载任务
    let tasks: Vec<DownloadTask> = uris_to_download
        .iter()
        .map(|url| {
            let relative_path = extract_relative_path(&args.url, url);
            DownloadTask {
                url: url.clone(),
                relative_path,
            }
        })
        .collect();

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
