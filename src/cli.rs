use clap::Parser;

/// 3D Tiles 数据下载工具
#[derive(Parser, Debug, Clone)]
#[command(name = "tiles-downloader")]
#[command(author = "Sogrey <suzhaoheng2006@163.com>")]
#[command(version = "0.1.0")]
#[command(about = "高性能 3D Tiles 数据下载工具")]
pub struct Args {
    /// 3dtiles 数据地址 (tileset.json 的 URL)
    #[arg(short, long)]
    pub url: String,

    /// 输出目录路径
    #[arg(short, long)]
    pub dir: String,

    /// 开始下载位置下标
    #[arg(short = 's', long, default_value = "0")]
    pub start: usize,

    /// 结束下载位置下标 (0 表示下载到结束)
    #[arg(short = 'e', long, default_value = "0")]
    pub end: usize,

    /// 线程数
    #[arg(short, long, default_value = "1")]
    pub threads: usize,

    /// Referer 请求头
    #[arg(short, long)]
    pub referer: Option<String>,

    /// 跳过 SSL 证书验证（不安全，仅用于测试）
    #[arg(short = 'k', long)]
    pub insecure: bool,
}

impl Args {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
