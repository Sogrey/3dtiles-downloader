use anyhow::Result;
use std::path::Path;
use tokio::fs;

/// 确保目录存在
pub async fn ensure_dir(path: &str) -> Result<()> {
    let p = Path::new(path);
    if !p.exists() {
        fs::create_dir_all(p).await?;
    }
    Ok(())
}

/// 从 URL 提取文件名
pub fn extract_filename(url: &str) -> String {
    let url_path = url::Url::parse(url)
        .map(|u| u.path().to_string())
        .unwrap_or_else(|_| url.to_string());

    if let Some(pos) = url_path.rfind('/') {
        url_path[pos + 1..].to_string()
    } else {
        url_path
    }
}

/// 从 URL 提取相对路径（基于 tileset.json 位置）
pub fn extract_relative_path(base_url: &str, url: &str) -> String {
    // 提取 base_url 的路径部分
    let base_path = if let Ok(parsed) = url::Url::parse(base_url) {
        let path = parsed.path();
        if let Some(pos) = path.rfind('/') {
            path[..pos].to_string()
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };

    // 提取 url 的路径部分
    let url_path = if let Ok(parsed) = url::Url::parse(url) {
        parsed.path().to_string()
    } else {
        url.to_string()
    };

    // 计算相对路径
    if url_path.starts_with(&base_path) {
        url_path[base_path.len() + 1..].to_string()
    } else {
        url_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_filename() {
        let url = "https://example.com/data/tileset.json";
        assert_eq!(extract_filename(url), "tileset.json");
    }

    #[test]
    fn test_extract_relative_path() {
        let base = "https://example.com/data/tileset.json";
        let url = "https://example.com/data/tiles/0/0/0.b3dm";
        assert_eq!(extract_relative_path(base, url), "tiles/0/0/0.b3dm");
    }
}
