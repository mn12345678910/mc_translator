<!-- 最後更新: 2026-04-15 | 對應版本: v1.0.2 -->

# 依賴告警處理指南

本文件整理目前專案在 RustSec / `cargo audit` 中會遇到的依賴告警，並說明哪些可以由本專案直接修復，哪些屬於上游 Tauri 生態鏈的暫時阻塞。

## 分類原則

依賴告警分為三類：

1. 可直接修復
   - 來自 `Cargo.toml` 或 `src-tauri/Cargo.toml` 的直接依賴。
   - 可以透過升級版本、調整 feature、替換 crate 解決。
2. 轉移依賴，可隨上游升級修復
   - 不是本專案直接引用，而是由 `tauri`、`tauri-utils`、`wry` 等上游帶入。
   - 本專案可做的動作是追蹤上游版本，並在新版本可用時儘快升級。
3. 轉移依賴，暫無安全 workaround
   - 不能安全地用 `[patch.crates-io]` 強行跨大版本替換。
   - 需要保留 issue、註記風險、等待上游。

## 當前結論

截至 2026-04-15，本專案使用：

- `tauri = 2.10.3`
- `tauri-utils = 2.8.3`
- `tauri-runtime-wry = 2.10.1`
- `wry = 0.54.4`（由 Tauri 生態鏈帶入）

本次已確認 `rand 0.9.2` 已從鎖檔移除，替換為 `rand 0.9.3`，因此針對 `rand 0.9.2` 的 advisory 已可視為已修復。

## 告警分類表

| 類型 | Issue | RustSec | 來源 | 現況 | 行動 |
| --- | --- | --- | --- | --- | --- |
| 已修復 | #46 | `RUSTSEC-2026-0097` (`rand 0.9.2`) | 舊版鎖檔 | 已升至 `rand 0.9.3` | 可關閉 |
| 上游轉移依賴 | #45 | `RUSTSEC-2026-0097` (`rand 0.8.5`) | `tauri-utils -> kuchikiki -> selectors/phf` | 仍存在 | 追蹤 `tauri-utils` |
| 上游轉移依賴 | #44 | `RUSTSEC-2026-0097` (`rand 0.7.3`) | `tauri-utils -> kuchikiki -> selectors/phf` | 仍存在 | 追蹤 `tauri-utils` |
| 上游轉移依賴 | #43 | `RUSTSEC-2024-0429` (`glib 0.18.5`) | `tauri -> tauri-runtime-wry -> wry/webkit2gtk/gtk` | 仍存在 | 追蹤 Tauri / Wry Linux 鏈 |
| 上游轉移依賴 | #42, #41, #40, #39, #38 | `unic-*` unmaintained | `tauri-utils -> urlpattern 0.3.0` | 仍存在 | 追蹤 `urlpattern` / `tauri-utils` |
| 上游轉移依賴 | #28 | `fxhash 0.2.1` unmaintained | `tauri-utils -> kuchikiki -> selectors` | 仍存在 | 追蹤 `tauri-utils` |
| 上游轉移依賴 | #37 | `proc-macro-error 1.0.4` unmaintained | `gtk/glib` macro 鏈 | 仍存在 | 追蹤 GTK/GLib 鏈 |
| 上游轉移依賴 | #27, #29, #30, #31, #32, #33, #34, #35, #36 | GTK3 unmaintained | `tauri -> tauri-runtime-wry -> gtk/webkit2gtk` | 仍存在 | 追蹤 Tauri / Wry Linux 鏈 |

## 實際依賴來源

### `rand 0.8.5` / `rand 0.7.3`

來自：

`tauri-utils -> kuchikiki -> selectors -> phf`

代表這不是本專案的直接 `rand` 使用，而是 HTML / selector 處理相關的上游 build/runtime 工具鏈。

### `unic-*`

來自：

`tauri-utils -> urlpattern 0.3.0 -> unic-*`

`urlpattern` crates.io 最新版已高於目前使用版本，但 `tauri-utils 2.8.3` 仍固定帶入 `0.3.0`。

### GTK / GLib / `proc-macro-error`

來自：

`tauri -> tauri-runtime-wry -> wry -> webkit2gtk / gtk / glib`

這組主要影響 Linux GUI 依賴鏈，不是本專案在業務邏輯中直接引用 GTK3。

## 處理流程

1. 先跑 `cargo audit --deny warnings`。
2. 再用 `cargo tree -i <crate> --workspace --target all` 找出來源。
3. 若為直接依賴，優先在本專案修復。
4. 若為轉移依賴，檢查上游最新版本是否已解。
5. 若上游尚未解決，在 `.cargo/audit.toml` 記錄忽略原因，並於 GitHub issue 留下來源與阻塞點。

## 不建議的做法

- 不要為了消除告警，對 `tauri-utils` 或 GTK 鏈做跨大版本 `[patch.crates-io]` 強制替換。
- 不要在未驗證 Tauri 相容性的情況下，手動覆蓋 `urlpattern`、`glib`、`wry` 的主版本。

這類修法很容易讓 GUI runtime 或 build-time codegen 出現不相容問題。

## 後續計畫

1. 定期檢查 `tauri`、`tauri-utils`、`tauri-runtime-wry`、`wry` 新版本。
2. 每次 Dependabot / `Cargo.lock` 更新後重新跑 `cargo audit` 與 `cargo tree` 分析。
3. 當上游釋出可消除 transitive advisory 的版本時，優先安排升級。
