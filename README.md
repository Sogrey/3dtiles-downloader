# 3DTiles Downloader (Rust)

高性能 3D Tiles 数据下载工具，使用 Rust 重写。

## 功能特性

- ✅ 解析 tileset.json 文件
- ✅ 递归获取所有资源文件
- ✅ 多线程并发下载
- ✅ 支持分段下载
- ✅ 断点续传
- ✅ 自动处理压缩格式（gzip、deflate、brotli）
- ✅ 进度显示
- ✅ 错误重试机制

## 安装

### 从源码编译

```bash
git clone git@github.com:Sogrey/3dtiles-downloader.git
cd 3dtiles-downloader
cargo build --release
```

编译后的二进制文件位于 `target/release/tiles_downloader.exe`

## 使用方法

### 基本用法

```bash
tiles_downloader --url <TILESET_URL> --dir <OUTPUT_DIR>
```

### 完整参数

```bash
tiles_downloader \
  --url https://example.com/tileset.json \
  --dir ./output \
  --threads 4 \
  --start 0 \
  --end 1000 \
  --referer "https://example.com"
```

### 参数说明

| 参数 | 缩写 | 必填 | 说明 | 默认值 |
|------|------|------|------|--------|
| `--url` | `-u` | 是 | 3dtiles 数据地址 | - |
| `--dir` | `-d` | 是 | 输出目录路径 | - |
| `--start` | `-s` | 否 | 开始下载位置下标 | 0 |
| `--end` | `-e` | 否 | 结束下载位置下标 | 总长度 |
| `--threads` | `-t` | 否 | 线程数 | 1 |
| `--referer` | `-r` | 否 | Referer 请求头 | - |
| `--insecure` | `-k` | 否 | 跳过 SSL 证书验证 | false |

### 示例

```bash
# 单线程下载
tiles_downloader -u https://lab.earthsdk.com/model/702aa950d03c11e99f7ddd77cbe22fea/tileset.json -d ./data

# 4线程下载
tiles_downloader -u https://example.com/tileset.json -d ./data -t 4

# 分段下载（下载前1000个文件）
tiles_downloader -u https://example.com/tileset.json -d ./data -s 0 -e 1000 -t 4

# 跳过 SSL 验证（用于测试环境或证书过期的服务器）
tiles_downloader -u https://example.com/tileset.json -d ./data -k

tiles_downloader --url https://stone.fren.ink/demo/data/tileset.json --dir download --threads 3 --insecure
```

## 与 Python 版本对比

| 特性 | Python 版本 | Rust 版本 |
|------|-------------|-----------|
| 性能 | 较慢 | 快 ⚡ |
| 内存 | 较高 | 低 💪 |
| 并发 | 多线程 | 异步 + 多线程 🚀 |
| 部署 | 需要 Python 环境 | 单文件可执行 📦 |
| 递归子 tileset | ✓ | ✓ ✅ |
| SSL 跳过 | ✗ | ✓ ✅ |
| 重试机制 | ✓ | ✓ (最多 3 次) ✅ |
| 进度显示 | ✓ | ✓ (带文件大小) ✅ |

## 输出说明

下载完成后，输出目录结构如下：

```
output/
├── tileset.json          # 主 tileset 文件
├── sub/                   # 子目录（如果有）
│   ├── tileset.json      # 子 tileset 文件
│   └── tiles/            # 瓦片数据
│       ├── 0-0.b3dm
│       └── 0-1.b3dm
└── tiles/                # 瓦片数据
    ├── 0-0.b3dm
    └── 0-1.glb
```

所有 tileset.json 和瓦片文件都会保持原有的目录结构。

## 开发

详见 [DEVELOPMENT.md](./DEVELOPMENT.md)

## 许可证

MIT License

## 致谢

原 Python 版本: [IKangXu/3dtilesdownloader](https://github.com/IKangXu/3dtilesdownloader)
