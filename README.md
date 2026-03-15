# Minecraft Mod Auto-Translator (mc_translator_rs)

> [!CAUTION]
> **免責聲明**
> 本工具雖使用微軟 DPAPI 對 API KEY 進行加密，但由於本工具 100% 使用 AI 開發，如有疑慮者請 **"不要"** 使用需要填入 API KEY 的服務商。

> [!IMPORTANT]
> **翻譯品質須知**
> 本工具只能用於快速翻譯，品質絕對沒辦法達到 100% 人工翻譯精準。請 **不要** 將本工具產出翻譯用於提交給模組作者。

這是一款為 Minecraft 整合包開發者設計的自動化翻譯工具，利用 LLM (Gemini, OpenAI, Ollama, DeepSeek, Mistral, DeepL) 技術，實現自動翻譯 JAR、JS、JSON 檔案。

## 🌟 核心功能

### 翻譯引擎
- **多服務商支援**：Gemini、OpenAI、DeepSeek、Mistral、Ollama、DeepL
- **動態模型列表**：自動從各服務商 API 獲取可用模型
- **批量翻譯**：可調整批次大小（1-300 行，預設 100），減少 API 呼叫次數
- **智慧過濾**：自動跳過技術性 ID（如 `tconstruct:broad_axe`）、布林值、純數字、snake_case 標識符

### 檔案處理
- 自動掃描 `mods/` 資料夾中的 JAR 檔案
- 支援 KubeJS（`.js`）和 PackMenu 腳本翻譯
- 支援獨立 `en_us.json` 檔案翻譯
- **輸出結構**：
  1. **LLMTranslator.zip (資源包壓縮檔)**
     只有來自 `mods` 資料夾內的 JAR 模組檔，並且符合標準 Minecraft 資源結構 (`assets/`) 的翻譯，才會被打包進這個 ZIP 檔中。這個檔案可以直接丟入 Minecraft 的 `resourcepacks` 資料夾使用。
     - **pack.mcmeta**: 程式會自動在 ZIP 根目錄生成此檔案，並根據你在 UI 選擇的版本填入對應的 `pack_format`。
     - **一般模組語言檔**: `assets/<modid>/lang/zh_tw.json`
     - **Patchouli 說明書 / 帕秋莉手冊**: `assets/<modid>/patchouli_books/<手冊名稱>/zh_tw/` (原本的 `en_us` 目錄會自動替換為 `zh_tw`，內部的 json 檔名維持不變)
  2. **實體資料夾與檔案 (直接輸出至你設定的目標資料夾)**
     針對獨立的腳本、非標準結構的資料夾（如 KubeJS）、或是獨立的 JSON 檔案，程式會直接在目標資料夾內保留原始路徑結構並輸出成實體檔案。
     - **特殊的獨立配置 (如 KubeJS, PackMenu)**: 不論是獨立選擇資料夾，還是存在於某些整合包架構中，只要開頭是 `kubejs/` 或 `packmenu/`，程式都會直接在輸出資料夾產生實體的目錄：
       - `kubejs/.../zh_tw.json`
       - `packmenu/.../zh_tw.json`
     - **獨立的 JSON 檔案**: 保留它在你原本選擇的相對層級。例如，如果你選擇翻譯某個 `en_us.json`，它會在輸出路徑對應的位置產生 `zh_tw.json`。
     - **獨立的 JS 腳本檔案**: 檔名維持原本的 JS 檔名不變 (例如 `recipes.js` 翻譯後還是輸出為 `recipes.js`)，並保留原有的相對資料夾路徑。

### 翻譯記憶體（字典）
- 已翻譯的字串自動儲存，加速後續翻譯並降低成本
- **增量儲存**：每個檔案翻譯完成後自動儲存，防止中途中斷資料遺失
- **進階字典管理器**：
  - 🔍 搜尋：即時過濾原文或翻譯
  - 📄 分頁：每頁 50 筆，支援翻頁導航
  - ✏ 編輯：直接修改翻譯內容
  - 🗑 刪除：移除單筆或清空全部
  - 📥 匯入 / 📤 匯出：JSON 格式匯入匯出
  - ⚠ 翻譯中鎖定：翻譯進行中自動鎖定編輯操作

### 暫停 / 繼續 / 停止
- **主動暫停**：翻譯中按 `⏸ 暫停` 即可暫停，按 `▶ 繼續` 恢復
- **Ollama 逾時自動暫停**：設定逾時秒數（1-300 秒，預設 60），逾時後自動暫停並儲存記憶體
- **停止**：可隨時中止翻譯流程

### 進度追蹤
- **檔案進度條**：顯示目前檔案的翻譯進度（n/total）
- **總進度條**：顯示已完成檔案數佔總數的比例（n/total [X%]）
- **即時日誌**：使用 TextEdit 高效能顯示，上限 1000 條

### 版本管理
- **動態版本列表**：從 Mojang API 自動獲取 Minecraft Release 版本
- **pack_format 自動映射**：選擇版本後自動填入對應的 pack_format
- **手動編輯**：也可直接修改 pack_format 數值

### 安全性
- API Key 使用 Windows DPAPI 技術進行硬體級加密

## 💻 需求環境

| 項目 | 需求 |
|------|------|
| **作業系統** | Windows 10/11 (使用 DPAPI 加密) |
| **網路連接** | 需可連接至 API；Ollama 需啟動本地伺服器 |
| **硬體建議** | 翻譯主要依賴網路 API，本機需求極低；Ollama 依模型而定 |

## 📖 文檔中心 (Documentation)

關於程式的詳細設計與規範，請參閱 [docs/](docs/) 目錄：

- **架構維度**: [架構概覽](docs/architecture/overview.md) (包含 [狀態管理](docs/architecture/state_management.md)、[錯誤處理](docs/architecture/error_handling.md))
- **邏輯維度**: [翻譯核心](docs/modules/translation_core.md) (包含 [檔案流水線](docs/modules/file_pipeline.md)、[術語系統](docs/modules/glossary_system.md)、[翻譯記憶體](docs/modules/translation_memory.md))
- **視覺維度**: [UI 規格](docs/ui/specs.md) (包含 [交互地圖](docs/ui/interactions.md))
- **維護維度**: [測試策略](docs/guides/testing_strategy.md)、[維護日誌](docs/guides/MAINTENANCE_LOG.md)、[Git 指南](docs/guides/GIT_GITHUB_GUIDE.md)

## 📖 操作說明

1. **設定 API**：
   - 啟動程式後，點擊右上角「⚙」齒輪圖示進入 API 設定
   - 選擇服務商及模型，並填入 API 金鑰（金鑰儲存後自動加密）
2. **選擇翻譯範圍**：
   - 「選擇檔案」翻譯特定檔案
   - 「選擇資料夾」選擇整個整合包根目錄
3. **設定輸出路徑**：
   - 點擊「輸出資料夾」選擇輸出位置（預設為工具所在目錄）
4. **調整參數**：
   - 批次大小（1-300）、Ollama 逾時秒數（1-300）
   - MC 版本 / pack_format、自訂翻譯提示
5. **開始翻譯**：
   - 點擊「▶ 開始翻譯」
   - 翻譯中可暫停（⏸）、繼續（▶）、停止（■）
   - 從日誌與進度條即時追蹤進度
6. **管理字典**：
   - 點擊「📖 開啟字典」進入翻譯記憶體管理器
   - 可搜尋、編輯、刪除、匯入匯出翻譯記錄
7. **套用翻譯**：
   - `LLMTranslator.zip` → 放入 `resourcepacks` 資料夾
   - `kubejs/` 等資料夾 → 放入指定輸出目錄
