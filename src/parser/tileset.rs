use serde::{Deserialize, Serialize};

/// 3D Tiles 根结构
#[derive(Debug, Serialize, Deserialize)]
pub struct Tileset {
    /// 资产信息
    pub asset: Asset,
    /// 几何误差阈值
    #[serde(default)]
    pub geometric_error: Option<f64>,
    /// 根节点
    pub root: Tile,
    /// 扩展字段
    #[serde(default)]
    pub extras: Option<serde_json::Value>,
    /// 扩展
    #[serde(default, rename = "extensions")]
    pub extensions: Option<serde_json::Value>,
}

/// 资产信息
#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    /// 版本号
    pub version: String,
    /// tileset 版本
    #[serde(default, rename = "tilesetVersion")]
    pub tileset_version: Option<String>,
    /// 扩展
    #[serde(default)]
    pub extensions: Option<serde_json::Value>,
    /// 扩展字段
    #[serde(default)]
    pub extras: Option<serde_json::Value>,
}

/// Tile 节点
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tile {
    /// 包围盒
    #[serde(default, rename = "boundingVolume")]
    pub bounding_volume: Option<BoundingVolume>,
    /// 视图体积
    #[serde(default, rename = "viewerRequestVolume")]
    pub viewer_request_volume: Option<BoundingVolume>,
    /// 几何误差阈值
    #[serde(default)]
    pub geometric_error: Option<f64>,
    /// 细化策略 (ADD / REPLACE)
    #[serde(default, rename = "refine")]
    pub refine: Option<String>,
    /// 内容
    #[serde(default)]
    pub content: Option<TileContent>,
    /// 子节点
    #[serde(default)]
    pub children: Vec<Tile>,
    /// 变换矩阵
    #[serde(default, rename = "transform")]
    pub transform: Option<Vec<f64>>,
    /// 扩展
    #[serde(default)]
    pub extensions: Option<serde_json::Value>,
    /// 扩展字段
    #[serde(default)]
    pub extras: Option<serde_json::Value>,
}

/// Tile 内容
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TileContent {
    /// 资源 URI
    pub uri: String,
    /// 包围盒
    #[serde(default, rename = "boundingVolume")]
    pub bounding_volume: Option<BoundingVolume>,
    /// 扩展
    #[serde(default)]
    pub extensions: Option<serde_json::Value>,
    /// 扩展字段
    #[serde(default)]
    pub extras: Option<serde_json::Value>,
}

/// 包围盒
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoundingVolume {
    /// 盒子 (12 个元素)
    #[serde(default, rename = "box")]
    pub box_: Option<Vec<f64>>,
    /// 区域 (6 个元素: west, south, east, north, minimum height, maximum height)
    #[serde(default)]
    pub region: Option<Vec<f64>>,
    /// 球体 (4 个元素: x, y, z, radius)
    #[serde(default)]
    pub sphere: Option<Vec<f64>>,
    /// 扩展
    #[serde(default)]
    pub extensions: Option<serde_json::Value>,
}

impl Tileset {
    /// 从 JSON 字符串解析
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let tileset: Tileset = serde_json::from_str(json)?;
        Ok(tileset)
    }

    /// 获取所有资源 URI（包括子 tileset）
    pub fn get_all_uris(&self, base_url: &str) -> Vec<String> {
        let mut uris = Vec::new();
        Self::collect_uris(&self.root, base_url, &mut uris);
        uris
    }

    /// 递归收集 URI
    fn collect_uris(tile: &Tile, base_url: &str, uris: &mut Vec<String>) {
        // 收集内容 URI
        if let Some(content) = &tile.content {
            let full_url = Self::resolve_url(base_url, &content.uri);
            uris.push(full_url);
        }

        // 递归处理子节点
        for child in &tile.children {
            Self::collect_uris(child, base_url, uris);
        }
    }

    /// 解析相对 URL
    fn resolve_url(base_url: &str, uri: &str) -> String {
        if uri.starts_with("http://") || uri.starts_with("https://") {
            return uri.to_string();
        }

        // 移除 base_url 的最后一部分
        let base = if let Some(pos) = base_url.rfind('/') {
            &base_url[..pos]
        } else {
            base_url
        };

        // 处理相对路径
        if uri.starts_with("./") {
            format!("{}/{}", base, &uri[2..])
        } else if uri.starts_with("../") {
            // 简化处理，实际可能需要更复杂的路径解析
            format!("{}/{}", base, uri)
        } else {
            format!("{}/{}", base, uri)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_tileset() {
        let json = r#"
        {
            "asset": {
                "version": "1.0"
            },
            "geometricError": 4096,
            "root": {
                "boundingVolume": {
                    "box": [0, 0, 0, 100, 0, 0, 0, 100, 0, 0, 0, 100]
                },
                "geometricError": 512,
                "content": {
                    "uri": "root.b3dm"
                },
                "children": []
            }
        }
        "#;

        let tileset = Tileset::from_json(json).unwrap();
        assert_eq!(tileset.asset.version, "1.0");
        assert!(tileset.root.content.is_some());
    }
}
