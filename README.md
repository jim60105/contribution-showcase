# contribution-showcase

一個 Rust CLI 工具，掃描多個 Git 儲存庫的提交紀錄，產生一頁式 HTML 貢獻總覽報告。

## 功能

- 讀取 TOML 設定檔（`showcase.toml`），支援多專案、篩選條件與輸出設定
- 解析 Conventional Commit 格式，依類型統計提交分佈
- 統計程式碼異動行數（新增／刪除）
- 收集 OpenSpec 提案清單
- 產生獨立 HTML 報告，包含：
  - 提交時間軸視覺化
  - Conventional Commit 類型分佈圖
  - 各專案統計資訊
  - 測試覆蓋率資訊

## 快速開始

### 前置需求

- [Rust](https://www.rust-lang.org/) 工具鏈（`cargo`）
- Git

### 安裝與建置

```bash
cargo build --release
```

編譯產物位於 `target/release/contribution-showcase`。

### 使用流程

1. 建立起始設定檔：

   ```bash
   contribution-showcase init
   ```

2. 編輯 `showcase.toml`，設定要掃描的儲存庫路徑與篩選條件。

3. 產生報告：

   ```bash
   contribution-showcase generate
   ```

   報告預設輸出至 `dist/index.html`。

## 設定檔

設定檔為 TOML 格式，結構如下（完整範例請參考 `showcase.example.toml`）：

```toml
# 報告標題（選填）
title = "My Contribution Showcase"

# 輸出設定（選填）
[output]
path = "dist/index.html"     # 預設值

# 全域篩選條件（選填）
[filters]
author = "Jane Doe"           # 依作者名稱篩選
since = "2025-01-01"          # 起始日期（YYYY-MM-DD）
until = "2025-06-30"          # 結束日期（YYYY-MM-DD）
types = ["feat", "fix"]       # 限定 Conventional Commit 類型
exclude_hashes = ["e3b0..."]  # 排除特定 commit SHA

# 專案清單（至少一筆）
[[projects]]
name = "my-backend"
path = "../my-backend"
description = "REST API server"
branch = "main"
coverage_command = "cargo llvm-cov --html"
coverage_result_path = "target/llvm-cov/html/index.html"
```

**必填欄位：** `name`、`path`。其餘欄位皆為選填。

## CLI 用法

### `generate` — 產生報告

```bash
contribution-showcase generate [OPTIONS]
```

| 選項 | 說明 |
|---|---|
| `-c, --config <FILE>` | 設定檔路徑（預設：`showcase.toml`） |
| `-o, --output <FILE>` | 輸出路徑（覆寫設定檔中的值） |
| `--author <NAME>` | 依作者篩選（覆寫設定檔中的值） |
| `--since <DATE>` | 起始日期 YYYY-MM-DD（覆寫設定檔中的值） |
| `--until <DATE>` | 結束日期 YYYY-MM-DD（覆寫設定檔中的值） |

### `init` — 建立起始設定檔

```bash
contribution-showcase init [OPTIONS]
```

| 選項 | 說明 |
|---|---|
| `-o, --output <FILE>` | 設定檔輸出路徑（預設：`showcase.toml`） |

## 開發

```bash
# 執行測試
cargo test

# 格式化程式碼
cargo fmt

# 靜態分析
cargo clippy
```

## 授權條款

本專案採用 [MIT License](LICENSE) 授權。
