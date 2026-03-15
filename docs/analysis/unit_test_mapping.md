# 模組功能與三項測試規範對應分析 (Unit Test Mapping)

本文件根據 `docs/guides/testing_strategy.md` 定義的 **「三項測試規則 (3-Test Rule)」**，針對專案內各核心模組的功能進行單元測試需求分析。

---

## 📋 三項測試規則定義

1.  **正常路徑測試 (Happy Path)**：驗證功能在標準、預期輸入下的正確性。
2.  **邊界值測試 (Edge Cases / UTF-8)**：
    *   驗證空輸入、極值、特殊字元。
    *   **強制要求**：包含 Unicode、表情符號等 UTF-8 測試。
3.  **強韌性與異常處理 (Robustness / Negative Cases)**：
    *   驗證錯誤輸入、損壞數據。
    *   防範無限迴圈（如遞迴、循環替換）。

---

## 📂 模組分析與測試對應

### 1. 文本處理與輔助工具 (`utils/`)

此類模組多為 **純邏輯函式 (Pure Functions)**，無副作用，最適合進行嚴格的單元測試。

#### 🔹 `text_processing.rs` (文本前/後處理)
*   **核心功能**：`validate_and_cleanup` (清理翻譯結果)、`preprocess_text` (預留預置符)、`detect_loop` (迴圈偵測)。
*   **🧪 測試映射**：
    *   **Happy Path**：
        *   驗證 `Translation: "..."` 能正確提取內部文字。
        *   驗證 `preprocess` 能正確將 `%s` 或 `§a` 替換為 `%%VAR_0%%`。
    *   **Edge Cases / UTF-8**：
        *   測試中文引號（`「」`、`『』`）的剝離。
        *   測試空字串 `""` 或僅包含空白的清理結果。
    *   **Robustness**：
        *   `detect_loop` 必須測試連續重複字串（如 `跳轉跳轉跳轉`）是否會被判定為 True。
        *   必須測試超長文字（>2000 字）的自動截斷防護。

#### 🔹 `helpers.rs` (輔助工具)
*   **核心功能**：`extract_display_path` (資產路徑提取)、`add_log` (日誌寫入)。
*   **🧪 測試映射**：
    *   **Happy Path**：
        *   給予 `assets/minecraft/lang/en_us.json`，驗證輸出 `{minecraft}/en_us.json`。
    *   **Edge Cases / UTF-8**：
        *   路徑包含中文字元、空白或特殊符號時的顯示。
    *   **Robustness**：
        *   給予不包含 `assets` 的路徑，驗證是否安全退化至 `file_name()`。

---

### 2. 術語與自動機 (`translation/glossary/`)

#### 🔹 `automaton.rs` (Aho-Corasick 匹配)
*   **核心功能**：`GlossaryAutomaton::extract` (提取匹配術語)。
*   **🧪 測試映射**：
    *   **Happy Path**：
        *   驗證輸入 `Apple` 能正確匹配到 `蘋果`（不區分大小寫）。
    *   **Edge Cases / UTF-8**：
        *   測試術語包含 ❄️ 表情符號或複合字元時的精確匹配。
    *   **Robustness**：
        *   **防呆測試**：設定循環術語（`A -> B` 且 `B -> A`），驗證連續提取時不會陷入死循環（Aho-Corasick 單次掃描應安全）。

---

### 3. 設定與辭典管理 (`config/`)

此類模組涉及 **I/O 操作**，測試須著重於檔案讀寫的安全性與容錯。

#### 🔹 `dictionary.rs` (辭典 IO)
*   **核心功能**：`load_dict` (載入)、`save_dict` (儲存)。
*   **🧪 測試映射**：
    *   **Happy Path**：
        *   儲存一個 `HashMap` 後，再次讀取，驗證內容一致。
    *   **Edge Cases / UTF-8**：
        *   字典內容包含複雜 JSON 轉義字元、多國語言時的讀寫一致性。
    *   **Robustness**：
        *   **損壞檔案測試**：若 `user.json` 內容為無效 JSON，驗證是否能安全退化至 `Default::default()`，而非崩潰。

#### 🔹 `encryption.rs` (密鑰加密)
*   **核心功能**：`encrypt_string` / `decrypt_string` (Windows DPAPI)。
*   **🧪 測試映射**：
    *   **Happy Path**：
        *   加密一段文字，解密後與原文相同。
    *   **Robustness**：
        *   解密一段隨機產生的無效 Base64 位元組，驗證是否會安全返回 `Err`，不引發記憶體安全問題。

---

### 4. 狀態管理 (`state/`)

此類模組多涉及 **變時狀態 (Shared State)** 與 **並發 (Concurrency)**。

#### 🔹 `app_state.rs` (全域狀態)
*   **核心功能**：`trigger_save` (觸發非同步儲存)。
*   **🧪 測試映射**：
    *   **Happy Path**：
        *   驗證呼叫 `trigger_save` 後，`save_tx` 通道能收到對應的 `ConfigPacket` 快照。
    *   **Robustness**：
        *   測試在高頻呼叫 `trigger_save` 且通道滿載時，系統是否會發生 Deadlock（死鎖）或 Blocking（阻塞）。

---

### 5. 翻譯引擎 (`translation/`)

#### 🔹 `engine.rs` (遞迴翻譯)
*   **核心功能**：`translate_json_recursive` (遞迴掃描)、`collect_translatable_strings` (收集待翻譯項)。
*   **🧪 測試映射**：
    *   **Happy Path**：
        *   驗證一般的 `String` 欄位能正確被推入 `pending` 列表。
    *   **Edge Cases / UTF-8**：
        *   驗證 `should_skip_key` 或 `should_skip_value` 能正確過濾不需翻譯的項（如純數字、特定格式）。
    *   **Robustness**：
        *   **無限遞迴防護**：傳入一個具有循環參照或極深層級的 `serde_json::Value`，驗證系統能安全報錯或停止，不耗盡堆疊 (Stack Overflow)。

---

## 💡 測試實施建議

1.  **善用 `tempfile` / 隔離**：針對 `config/` 模組，測試時應使用臨時目錄，避免污染本機 `dicts/` 下的真實數據。
2.  **隔離外部 API**：針對 `engine.rs` 與 API 連線，應使用整合測試 (`tests/`) 並加上 `#[ignore]` 標記，單元測試應專注於**資料結構轉換與過濾邏輯**。
