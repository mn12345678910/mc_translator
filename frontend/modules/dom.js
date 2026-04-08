// frontend/modules/dom.js
// 集中管理所有 DOM 元素查詢，消除重複的 getElementById 呼叫

export const dom = {
    // API 設定
    get apiProvider() {
        return document.getElementById('api-provider');
    },
    get apiKey() {
        return document.getElementById('api-key');
    },
    get selectedModel() {
        return document.getElementById('selected-model');
    },
    get ollamaUrl() {
        return document.getElementById('ollama-url');
    },
    get apiBaseUrl() {
        return document.getElementById('api-base-url');
    },

    // 翻譯參數
    get batchSize() {
        return document.getElementById('batch-size');
    },
    get batchMaxChars() {
        return document.getElementById('batch-max-chars');
    },
    get timeoutSec() {
        return document.getElementById('timeout-sec');
    },
    get packFormat() {
        return document.getElementById('pack-format');
    },

    // 語言
    get uiLang() {
        return document.getElementById('ui-lang');
    },
    get sourceLang() {
        return document.getElementById('source-lang');
    },
    get targetLang() {
        return document.getElementById('target-lang');
    },

    // 路徑
    get inputPath() {
        return document.getElementById('input-path');
    },
    get outputDir() {
        return document.getElementById('output-dir');
    },

    // Prompt
    get systemPrompt() {
        return document.getElementById('system-prompt');
    },
    get userPrompt() {
        return document.getElementById('user-prompt');
    },

    // 開關
    get chkGlossaryPriority() {
        return document.getElementById('chk-glossary-priority');
    },
    get chkSkipJson() {
        return document.getElementById('chk-skip-json');
    },
    get chkSkipJs() {
        return document.getElementById('chk-skip-js');
    },
    get chkSkipJar() {
        return document.getElementById('chk-skip-jar');
    },
    get chkSkipBook() {
        return document.getElementById('chk-skip-book');
    },
    get chkLlmLog() {
        return document.getElementById('chk-llm-log');
    },
    get chkDebugLog() {
        return document.getElementById('chk-debug-log');
    },
    get chkDebugTools() {
        return document.getElementById('chk-debug-tools');
    },
    get chkFastConvert() {
        return document.getElementById('chk-fast-convert');
    },
    get chkBtnRounding() {
        return document.getElementById('chk-btn-rounding');
    },
    get chkPulse() {
        return document.getElementById('chk-pulse');
    },

    // 其他
    get excludedPaths() {
        return document.getElementById('excluded-paths');
    },
    get btnTranslate() {
        return document.getElementById('btn-translate');
    },
    get btnPause() {
        return document.getElementById('btn-pause');
    },
    get btnResume() {
        return document.getElementById('btn-resume');
    },
    get btnStop() {
        return document.getElementById('btn-stop');
    },

    // 群組
    get ollamaUrlGroup() {
        return document.getElementById('ollama-url-group');
    },
    get apiKeyGroup() {
        return document.getElementById('api-key-group');
    },
    get apiBaseUrlGroup() {
        return document.getElementById('api-base-url-group');
    },
    get groupFastConvert() {
        return document.getElementById('group-fast-convert');
    },

    // 樣式控制
    get fontSize() {
        return document.getElementById('font-size');
    },
    get btnRoundingValue() {
        return document.getElementById('btn-rounding-value');
    },
    get pulseSpeed() {
        return document.getElementById('pulse-speed');
    },
    get progressStyle() {
        return document.getElementById('progress-style');
    },
    get progressBar() {
        return document.getElementById('progress-bar');
    },
    get batchProgressBar() {
        return document.getElementById('batch-progress-bar');
    },

    // 導航按鈕
    get btnNavApi() {
        return document.getElementById('btn-nav-api');
    },
    get btnNavDict() {
        return document.getElementById('btn-nav-dict');
    },
    get btnNavPalette() {
        return document.getElementById('btn-nav-palette');
    },
    get btnNavTheme() {
        return document.getElementById('btn-nav-theme');
    },
    get btnNavDev() {
        return document.getElementById('btn-nav-dev');
    },

    // 瀏覽按鈕
    get btnBrowseFile() {
        return document.getElementById('btn-browse-file');
    },
    get btnBrowseDir() {
        return document.getElementById('btn-browse-dir');
    },
    get btnBrowseOutput() {
        return document.getElementById('btn-browse-output');
    },
    get btnBrowseOutputOpen() {
        return document.getElementById('btn-browse-output-open');
    },
    get btnRestoreApi() {
        return document.getElementById('btn-restore-api');
    },
    get btnRestoreDev() {
        return document.getElementById('btn-restore-dev');
    },
    get btnRestorePalette() {
        return document.getElementById('btn-restore-palette');
    },
    get btnPaletteClearItem() {
        return document.getElementById('btn-palette-clear-item');
    },

    // 字典管理
    get dictSearch() {
        return document.getElementById('dict-search');
    },
    get dictInputKey() {
        return document.getElementById('dict-input-key');
    },
    get dictInputValue() {
        return document.getElementById('dict-input-value');
    },
    get headerDictMgr() {
        return document.getElementById('header-dict-mgr');
    },
    get btnDictOpenJson() {
        return document.getElementById('btn-dict-open-json');
    },
    get btnDictAdd() {
        return document.getElementById('btn-dict-add');
    },
    get btnDictReplace() {
        return document.getElementById('btn-dict-replace');
    },
    get btnDictClear() {
        return document.getElementById('btn-dict-clear');
    },
    get btnDictImport() {
        return document.getElementById('btn-dict-import');
    },
    get btnDictExport() {
        return document.getElementById('btn-dict-export');
    },
    get pageInfo() {
        return document.getElementById('page-info');
    },
    get pagePrev() {
        return document.getElementById('page-prev');
    },
    get pageNext() {
        return document.getElementById('page-next');
    },
    get dictTableContainer() {
        return document.getElementById('dict-table-container');
    },
    get dictUserControls() {
        return document.getElementById('dict-user-controls');
    },
    get tabUser() {
        return document.getElementById('tab-user');
    },
    get tabOfficial() {
        return document.getElementById('tab-official');
    },

    // 調色盤
    get paletteTargetType() {
        return document.getElementById('palette-target-type');
    },
    get paletteTargetItem() {
        return document.getElementById('palette-target-item');
    },
    get paletteProperty() {
        return document.getElementById('palette-property');
    },
    get palettePropertyGroup() {
        return document.getElementById('palette-property-group');
    },
    get paletteColorGroup() {
        return document.getElementById('palette-color-group');
    },
    get paletteNumberGroup() {
        return document.getElementById('palette-number-group');
    },
    get paletteClearGroup() {
        return document.getElementById('palette-clear-group');
    },
    get paletteNumber() {
        return document.getElementById('palette-number');
    },
    get paletteColor() {
        return document.getElementById('palette-color');
    },
    get labelPaletteNumber() {
        return document.getElementById('label-palette-number');
    },
    get labelPaletteColor() {
        return document.getElementById('label-palette-color');
    },

    // 狀態標籤
    get statusText() {
        return document.getElementById('status-text');
    },
    get batchStatusText() {
        return document.getElementById('batch-status-text');
    },
    get currentStatusLabel() {
        return document.getElementById('current-status-label');
    },

    // 除錯
    get debugRenderedCount() {
        return document.getElementById('debug-rendered-count');
    },
    get debugScrollLocked() {
        return document.getElementById('debug-scroll-locked');
    },
    get debugTotalLogs() {
        return document.getElementById('debug-total-logs');
    },
    get debugMemoryEst() {
        return document.getElementById('debug-memory-est');
    },

    // 顏色選擇器
    get colorBg() {
        return document.getElementById('color-bg');
    },
    get colorText() {
        return document.getElementById('color-text');
    },
    get colorAccent() {
        return document.getElementById('color-accent');
    },
    get colorDanger() {
        return document.getElementById('color-danger');
    },
};
