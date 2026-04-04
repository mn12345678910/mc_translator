const fs = require('fs');
const path = require('path');

// 設定路徑
const RUST_SETTINGS_PATH = path.join(__dirname, '../src/config/settings.rs');
const JS_TEST_CONFIG_PATH = path.join(__dirname, '../tests/frontend/config.test.js');

const RUST_DEFAULT_FN_MAP = {
    default_api_provider: '無',
    default_ollama_url: 'http://localhost:11434',
    default_user_prompt:
        '你是一位專業的 Minecraft 模組翻譯員。現在請將以下模組字串翻譯為「繁體中文 (zh_tw)」。\n保持專業的遊戲術語風格（如方塊、實體、附魔）。',
    default_system_prompt: '',
    default_batch_size: 150,
    default_batch_max_chars: 3500,
    default_timeout: 60,
    default_glossary_priority: 'official',
    default_source_lang: 'en_us',
    default_target_lang: 'zh_tw',
    default_ui_lang: 'zh_tw',
    default_pack_format: 15,
    default_true: true,
    default_false: false,
};

function resolveRustDefaultFn(name) {
    if (RUST_DEFAULT_FN_MAP[name] !== undefined) return RUST_DEFAULT_FN_MAP[name];
    return name;
}

function parseRustDefaults(filePath) {
    const content = fs.readFileSync(filePath, 'utf8');
    const defaults = {};

    const defaultMatch = content.match(
        /impl Default for AppConfig {[\s\S]*?fn default\(\) -> Self {([\s\S]*?)\n    }\n}/
    );
    if (!defaultMatch) {
        console.error('❌ 無法在 settings.rs 中找到 AppConfig 的 Default 實作');
        return null;
    }

    const selfBlock = defaultMatch[1];
    const lines = selfBlock.split('\n');
    lines.forEach((line) => {
        const match = line.match(/^\s*([a-z_]+):\s*(.*?),$/);
        if (match) {
            let key = match[1];
            let value = match[2].trim();

            const fnMatch = value.match(/^([a-z_]+)\(\)$/);
            if (fnMatch) {
                value = resolveRustDefaultFn(fnMatch[1]);
            } else if (value === 'String::new()' || value === '"".to_string()') {
                value = '';
            } else if (value.includes('.to_string()')) {
                value = value.replace('.to_string()', '').replace(/"/g, '');
            } else if (value.startsWith('"') && value.endsWith('"')) {
                value = value.replace(/"/g, '');
            } else if (value === 'DEFAULT_PROMPT') {
                value =
                    '你是一位專業的 Minecraft 模組翻譯員。現在請將以下模組字串翻譯為「繁體中文 (zh_tw)」。\n保持專業的遊戲術語風格（如方塊、實體、附魔）。';
            } else if (value === 'DEFAULT_SYSTEM_PROMPT') {
                value = '';
            } else if (value === 'false') {
                value = false;
            } else if (value === 'true') {
                value = true;
            } else if (!isNaN(value)) {
                value = Number(value);
            }

            if (key !== 'api_key' && key !== 'excluded_paths' && key !== 'user_prompt' && key !== 'system_prompt') {
                defaults[key] = value;
            }
        }
    });

    return defaults;
}

function parseJsMockDefaults(filePath) {
    const content = fs.readFileSync(filePath, 'utf8');

    // 尋找 mockDefaultConfig 的定義
    const mockMatch = content.match(/const mockDefaultConfig = {([\s\S]*?)};/);
    if (!mockMatch) {
        console.error('❌ 無法在 config.test.js 中找到 mockDefaultConfig');
        return null;
    }

    const objBlock = mockMatch[1];
    const lines = objBlock.split('\n');
    const mockDefaults = {};

    lines.forEach((line) => {
        // 範例: batch_size: 150,
        const match = line.match(/^\s*([a-z_]+):\s*(.*?),?$/);

        if (match) {
            let key = match[1];
            let value = match[2].trim();

            if (value.startsWith("'") || value.startsWith('"')) {
                value = value.replace(/['"]/g, '');
            } else if (value === 'false') {
                value = false;
            } else if (value === 'true') {
                value = true;
            } else if (!isNaN(value)) {
                value = Number(value);
            }

            if (key !== 'api_key' && key !== 'excluded_paths' && key !== 'user_prompt' && key !== 'system_prompt') {
                mockDefaults[key] = value;
            }
        }
    });

    return mockDefaults;
}

function syncCheck() {
    console.log('🔍 正在檢查 Rust 與 JS Mock 的預設配置同步情況...');

    const rustDefaults = parseRustDefaults(RUST_SETTINGS_PATH);
    const jsDefaults = parseJsMockDefaults(JS_TEST_CONFIG_PATH);

    if (!rustDefaults || !jsDefaults) {
        process.exit(1);
    }

    let hasError = false;
    const allKeys = new Set([...Object.keys(rustDefaults), ...Object.keys(jsDefaults)]);

    allKeys.forEach((key) => {
        if (!(key in rustDefaults)) {
            console.warn(`⚠️ [JS 獨有]: ${key} = ${jsDefaults[key]} (Rust 中無此預設值，可能已刪除)`);
        } else if (!(key in jsDefaults)) {
            console.error(`❌ [JS 缺失]: ${key} (Rust 中定義了該預設值，但前端測試 Mock 遺漏了)`);
            hasError = true;
        } else if (rustDefaults[key] !== jsDefaults[key]) {
            console.error(`❌ [數值不一致]: ${key}`);
            console.error(`   - Rust: ${rustDefaults[key]}`);
            console.error(`   - JS Mock: ${jsDefaults[key]}`);
            hasError = true;
        }
    });

    if (hasError) {
        console.error(
            '\n🛑 同步檢查失敗！請確保 tests/frontend/config.test.js 中的 mockDefaultConfig 與 src/config/settings.rs 保持一致。'
        );
        process.exit(1);
    } else {
        console.log('✅ 同步檢查通過！Rust 與 JS Mock 的關鍵預設值完全一致。');
        process.exit(0);
    }
}

syncCheck();
