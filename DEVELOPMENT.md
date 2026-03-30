# Tiles Downloader - Rust 版本开发计划

## 项目概述

将 Python 版本的 3dtiles-downloader 使用 Rust 重写，实现高性能、内存安全的 3D Tiles 数据下载工具。

**项目名称**: `tiles-downloader`

**作者**: Sogrey <suzhaoheng2006@163.com>

**仓库**: git@github.com:Sogrey/3dtiles-downloader.git

### 原项目地址
- GitHub: https://github.com/IKangXu/3dtilesdownloader
- 语言: Python 3.7+
- 功能: 下载在线 3dtiles 数据，支持多线程、分段下载、断点续传

---

## 功能需求

### 核心功能
- [x] 解析 tileset.json 文件
- [x] 递归获取所有子 tileset 和资源文件
- [x] 多线程并发下载
- [x] 支持分段下载（指定起始和结束位置）
- [x] 支持断点续传
- [x] 自动处理压缩格式（gzip、deflate、brotli）
- [x] 进度显示
- [x] 错误重试机制

### 命令行参数
| 参数 | 缩写 | 必填 | 说明 | 默认值 |
|------|------|------|------|--------|
| `--url` | `-u` | 是 | 3dtiles 数据地址 | - |
| `--dir` | `-d` | 是 | 输出目录路径 | - |
| `--start` | `-s` | 否 | 开始下载位置下标 | 0 |
| `--end` | `-e` | 否 | 结束下载位置下标 | 总长度 |
| `--threads` | `-t` | 否 | 线程数 | 1 |
| `--referer` | `-r` | 否 | Referer 请求头 | - |

---

## 技术栈

### 核心依赖
| 库名 | 用途 | 版本 |
|------|------|------|
| `tokio` | 异步运行时 | ^1.x |
| `reqwest` | HTTP 客户端 | ^0.12 |
| `serde` / `serde_json` | JSON 序列化/反序列化 | ^1.x |
| `clap` | 命令行参数解析 | ^4.x |
| `indicatif` | 进度条显示 | ^0.17 |
| `anyhow` | 错误处理 | ^1.x |
| `flate2` | gzip 解压 | ^1.x |
| `brotli` | brotli 解压 | ^6.x |

---

## 项目结构

```
tiles-downloader/
├── Cargo.toml              # 项目配置
├── DEVELOPMENT.md          # 开发计划文档（本文件）
├── README.md               # 项目说明
└── src/
    ├── main.rs             # 入口文件
    ├── cli.rs              # 命令行参数定义
    ├── lib.rs              # 库入口
    ├── downloader/         # 下载模块
    │   ├── mod.rs
    │   ├── http.rs         # HTTP 请求封装
    │   ├── task.rs         # 下载任务管理
    │   └── worker.rs       # 多线程工作器
    ├── parser/             # 解析模块
    │   ├── mod.rs
    │   ├── tileset.rs      # tileset.json 解析
    │   └── resource.rs     # 资源文件解析
    └── utils/              # 工具模块
        ├── mod.rs
        ├── file.rs         # 文件操作
        ├── decompress.rs   # 解压工具
        └── progress.rs     # 进度显示
```

---

## 模块设计

### 1. CLI 模块 (`cli.rs`)
- 使用 `clap` 定义命令行参数
- 参数验证和默认值处理

### 2. Parser 模块 (`parser/`)
#### tileset.rs
- 解析 `tileset.json` 文件结构
- 定义数据结构：
  ```rust
  struct Tileset {
      asset: Asset,
      geometric_error: f64,
      root: Tile,
  }
  
  struct Tile {
      bounding_volume: BoundingVolume,
      geometric_error: f64,
      content: Option<TileContent>,
      children: Vec<Tile>,
  }
  ```
- 递归遍历获取所有资源 URL

#### resource.rs
- 提取 b3dm、b3dm、i3dm、pnts 等资源文件路径
- 生成完整下载 URL 列表

### 3. Downloader 模块 (`downloader/`)
#### http.rs
- 异步 HTTP 请求封装
- 支持 Referer 等自定义 Header
- 超时设置
- 重试机制

#### task.rs
- 下载任务队列管理
- 任务调度
- 进度跟踪

#### worker.rs
- 多线程下载工作器
- 使用 `tokio::task::spawn_blocking` 或 `rayon` 实现并发
- 支持分段下载（start/end）

### 4. Utils 模块 (`utils/`)
#### file.rs
- 创建目录结构
- 保存文件
- 检查文件是否存在（断点续传）

#### decompress.rs
- 自动检测并解压 gzip、deflate、brotli 格式

#### progress.rs
- 使用 `indicatif` 显示进度条
- 显示下载速度、剩余时间

---

## 开发计划

### 第一阶段：基础框架（预计 2-3 小时）
1. 初始化项目结构
2. 配置 `Cargo.toml` 依赖
3. 实现 CLI 参数解析
4. 实现 tileset.json 基础解析

### 第二阶段：核心下载功能（预计 3-4 小时）
1. 实现 HTTP 请求模块
2. 实现文件下载和保存
3. 实现多线程下载
4. 实现进度显示

### 第三阶段：高级功能（预计 2-3 小时）
1. 实现分段下载
2. 实现断点续传
3. 实现压缩格式自动解压
4. 实现错误重试机制

### 第四阶段：优化和测试（预计 2 小时）
1. 错误处理优化
2. 性能优化
3. 实际场景测试
4. 文档完善

---

## 关键技术点

### 1. 3D Tiles 数据结构
- tileset.json 是树形结构，需要递归解析
- 资源文件类型：.b3dm、.glb、.i3dm、.pnts、.cmpt
- 可能包含外部 tileset.json（已实现递归处理）
- 使用 `HashSet` 避免重复处理同一个 tileset

### 2. 多线程下载
- 使用 `tokio` 异步运行时
- 使用 `Arc<Mutex<Vec<String>>>` 共享资源列表
- 使用 `JoinSet` 管理异步任务
- 使用 `Semaphore` 控制并发数

### 3. 断点续传
- 下载前检查文件是否存在
- 已存在则跳过下载
- 保存所有 tileset.json 文件

### 4. 压缩处理
- reqwest 自动处理 gzip、deflate、brotli
- 无需手动解压

### 5. 错误处理和重试
- 每个文件最多重试 3 次
- 重试间隔 2 秒
- 使用 `anyhow` 提供详细错误上下文

### 6. SSL 证书验证
- 支持 `--insecure` 跳过 SSL 验证
- 用于测试环境或证书过期的服务器

---

## 数据结构设计

### Tileset JSON 结构
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tileset {
    pub asset: Asset,
    #[serde(default)]
    pub geometric_error: Option<f64>,
    pub root: Tile,
    #[serde(default)]
    pub extras: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub version: String,
    #[serde(default)]
    pub tileset_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tile {
    #[serde(default)]
    pub bounding_volume: Option<BoundingVolume>,
    #[serde(default)]
    pub geometric_error: Option<f64>,
    #[serde(default)]
    pub content: Option<TileContent>,
    #[serde(default)]
    pub children: Vec<Tile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileContent {
    pub uri: String,
    #[serde(default)]
    pub bounding_volume: Option<BoundingVolume>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoundingVolume {
    #[serde(default)]
    pub box_: Option<Vec<f64>>,
    #[serde(default)]
    pub region: Option<Vec<f64>>,
    #[serde(default)]
    pub sphere: Option<Vec<f64>>,
}
```

---

## 开发优先级

### P0 - 必须实现
- tileset.json 解析
- 单线程下载
- 文件保存

### P1 - 重要功能
- 多线程下载
- 进度显示
- 错误处理

### P2 - 增强功能
- 分段下载
- 断点续传
- 压缩处理

---

## 测试策略

### 单元测试
- tileset.json 解析测试
- URL 生成测试
- 文件路径处理测试

### 集成测试
- 完整下载流程测试
- 多线程下载测试
- 断点续传测试

### 测试数据
使用公开的 3D Tiles 数据源：
```
https://lab.earthsdk.com/model/702aa950d03c11e99f7ddd77cbe22fea/tileset.json
```

---

## 参考资料

- [3D Tiles 规范](https://github.com/CesiumGS/3d-tiles)
- [Rust 异步编程](https://rust-lang.github.io/async-book/)
- [Tokio 文档](https://docs.rs/tokio/)
- [Reqwest 文档](https://docs.rs/reqwest/)
