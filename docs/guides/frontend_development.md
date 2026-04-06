<!-- 最後更新: 2026-04-06 | 對應版本: v1.0.0 -->

# 前端開發指南

本指南說明前端架構、開發環境設定與常用工具。

## 前端架構

```
frontend/
├── index.html          # 主頁面
├── main.js             # 入口：初始化所有模組
├── style.css           # 全域樣式
└── modules/
    ├── dom.js          # DOM 元素集中管理（80+ 個 getter）
    ├── state.js        # 全域狀態（currentConfig, currentLabels, currentStyle）
    ├── config.js       # API 設定模組
    ├── i18n.js         # 國際化模組
    ├── style.js        # 樣式系統
    ├── translation.js  # 翻譯控制模組
    ├── dictionary.js   # 字典管理模組
    ├── virtual_log.js  # 虛擬化日誌查看器
    ├── mock.js         # 開發環境 Mock 工具
    ├── devtools.js     # 開發者工具
    ├── palette.js      # 調色盤模組
    └── utils.js        # 工具函數
```

## 開發環境設定

### 啟動開發伺服器

```bash
pnpm dev
```

這會啟動 Vite 開發伺服器，支援 HMR（熱模組替換）。

### Mock 工具

開發環境下，`mock.js` 會自動攔截 `window.__TAURI__.core.invoke` 呼叫，返回模擬資料：

```javascript
// 自動載入（main.js）
if (import.meta.env?.DEV) {
    const { initMockTools } = await import('./modules/mock.js');
    await initMockTools();
}
```

新增 Mock 指令時，需同步更新 `allMockCommands` 陣列。

## DOM 模組 (`dom.js`)

所有 `document.getElementById(...)` 查詢已集中到 `dom.js`：

```javascript
import { dom } from './dom.js';

// 使用方式
if (dom.apiProvider) dom.apiProvider.value = 'Gemini';
if (dom.btnTranslate) dom.btnTranslate.disabled = true;
```

**優點：**

- 使用 getter 確保 SPA 環境下 DOM 元素被重新建立後仍能正確取得
- 消除 240+ 次重複的 `getElementById` 呼叫
- 集中管理，易於維護

**新增元素：**
在 `dom.js` 中加入新的 getter：

```javascript
export const dom = {
    // ... 現有元素 ...
    get myNewElement() {
        return document.getElementById('my-new-element');
    },
};
```

## 樣式系統

### 顏色套用

`style.js` 的 `applyColors()` 函數負責將 `StyleConfig` 套用為 CSS 變數：

```javascript
import { applyColors } from './style.js';

applyColors(state.currentStyle);
```

### CSS 變數

所有顏色以 CSS 變數形式定義在 `:root`：

```css
:root {
    --bg-color: rgb(30, 30, 35);
    --text-color: rgb(200, 160, 100);
    --accent-color: rgb(212, 175, 55);
    --font-size: 15px;
    --border-radius: 4px;
}
```

### 實例覆寫

`StyleConfig.instance_overrides` 可針對單一元件覆寫主題顏色或圓角：

```javascript
state.currentStyle.instance_overrides['btn-translate'] = {
    dark_bg: [255, 0, 0],
    light_bg: [200, 200, 200],
    rounding: 8.0,
};
applyColors(state.currentStyle);
```

## 狀態管理

全域狀態存放在 `state.js`：

```javascript
import { state } from './state.js';

state.currentConfig = { ... };
state.currentLabels = { ... };
state.currentStyle = { ... };
```

## i18n 系統

### 載入流程

```
loadUiLangs() → updateUiLanguage() → 更新所有 data-i18n 屬性
```

### 使用 data-i18n 屬性

```html
<h1 data-i18n="app_title">Minecraft 翻譯工具</h1>
<input data-i18n-placeholder="placeholder_search" placeholder="搜尋..." />
<button data-i18n-title="btn_translate" title="開始翻譯">翻譯</button>
```

### 新增翻譯標籤

在 `i18n_assets/gui/en_us.json` 和 `i18n_assets/gui/zh_tw.json` 中加入：

```json
{
    "my_new_label": "My New Label"
}
```

## 日誌系統

### VirtualLogViewer

高效能虛擬化日誌查看器，只渲染可視區域內的條目：

```javascript
import { VirtualLogViewer } from './modules/virtual_log.js';

window.__logViewer = new VirtualLogViewer('log-output', {
    onUpdate: (stats) => {
        console.log(`Rendered: ${stats.rendered}, Total: ${stats.total}`);
    },
});
```

### 新增日誌條目

```javascript
import { appendLog } from './modules/utils.js';

appendLog({
    level: 'Info',
    message: '翻譯完成',
    timestamp: Date.now(),
});
```

## 測試

```bash
pnpm test        # 執行所有前端測試
pnpm vitest run  # 執行 Vitest 測試
```
