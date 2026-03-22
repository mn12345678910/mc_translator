import { describe, it, expect, beforeEach, beforeAll, vi } from 'vitest';

// Mock 外部模組防止交互副作用 (提升至頂層)
vi.mock('../../frontend/modules/utils.js', () => ({
    appendLog: vi.fn()
}));
vi.mock('../../frontend/modules/i18n.js', () => ({
    updateUiLanguage: vi.fn()
}));

describe('config.js 設定管理模組', () => {
    let mockInvoke;
    let configModule;
    let stateModule;
    let utilsModule;
    let i18nModule;

    beforeAll(async () => {
        // 1. Mock Tauri API
        mockInvoke = vi.fn();
        globalThis.window = {
            __TAURI__: {
                core: { invoke: mockInvoke }
            }
        };

        // 2. 動態載入
        configModule = await import('../../frontend/modules/config.js');
        stateModule = await import('../../frontend/modules/state.js');
        utilsModule = await import('../../frontend/modules/utils.js');
        i18nModule = await import('../../frontend/modules/i18n.js');
    });

    beforeEach(() => {
        // 模擬完整的 DOM 結構
        document.body.innerHTML = `
            <select id="api-provider">
                <option value="Gemini">Gemini</option>
                <option value="Ollama">Ollama</option>
            </select>
            <input id="api-key" type="password" />
            <select id="selected-model"></select>
            <input id="ollama-url" />
            <input id="batch-size" />
            <input id="batch-max-chars" />
            <input id="timeout-sec" />
            <input id="pack-format" />
            <input id="chk-glossary-priority" type="checkbox" />
            <select id="ui-lang">
                <option value="zh_tw">zh_tw</option>
            </select>
            <input id="output-dir" />
            <textarea id="system-prompt"></textarea>
            <textarea id="user-prompt"></textarea>
            <input id="chk-skip-json" type="checkbox" />
            <input id="chk-skip-js" type="checkbox" />
            <input id="chk-skip-jar" type="checkbox" />
            <input id="chk-skip-book" type="checkbox" />
            <input id="chk-llm-log" type="checkbox" />
            <input id="input-path" />
            <div id="ollama-url-group" style="display:none"></div>
            <div id="api-key-group" style="display:block"></div>
            <button id="btn-translate"></button>
        `;

        // 重設 Mock
        mockInvoke.mockReset();
        vi.clearAllMocks();

        // 重設 State
        stateModule.state.currentConfig = {};
        stateModule.state.currentLabels = {};
    });

    it('loadConfig 應該從後端獲取參數並正確填入 DOM', async () => {
        const fakeConfig = {
            api_provider: 'Ollama',
            path: '/test/input',
            ollama_url: 'http://localhost:12345',
            batch_size: 200,
            batch_max_chars: 4000,
            timeout: 90,
            pack_format: 12,
            glossary_priority: 'user',
            ui_lang: 'zh_tw',
            output_dir: '/test/output',
            system_prompt: 'System rule',
            user_prompt: 'User prompt',
            skip_json: true,
            skip_js: true,
            skip_jar: false,
            skip_book: false,
            enable_llm_log: true,
            model: 'llama3'
        };

        mockInvoke.mockImplementation(async (cmd, args) => {
            if (cmd === 'get_config') return fakeConfig;
            if (cmd === 'get_api_key_cmd') return 'test-key-123';
            if (cmd === 'get_models_from_provider') return ['llama3', 'mistral'];
            return null;
        });

        await configModule.loadConfig();

        // 驗證 DOM 是否帶入正確數值
        expect(document.getElementById('api-provider').value).toBe('Ollama');
        expect(document.getElementById('api-key').value).toBe('test-key-123');
        expect(document.getElementById('input-path').value).toBe('/test/input');
        expect(document.getElementById('ollama-url').value).toBe('http://localhost:12345');
        expect(document.getElementById('batch-size').value).toBe('200');
        expect(document.getElementById('batch-max-chars').value).toBe('4000');
        expect(document.getElementById('timeout-sec').value).toBe('90');
        expect(document.getElementById('pack-format').value).toBe('12');
        expect(document.getElementById('chk-glossary-priority').checked).toBe(true);
        expect(document.getElementById('output-dir').value).toBe('/test/output');
        expect(document.getElementById('system-prompt').value).toBe('System rule');
        expect(document.getElementById('user-prompt').value).toBe('User prompt');
        
        expect(document.getElementById('chk-skip-json').checked).toBe(true);
        expect(document.getElementById('chk-skip-js').checked).toBe(true);
        expect(document.getElementById('chk-skip-jar').checked).toBe(false);
        expect(document.getElementById('chk-llm-log').checked).toBe(true);

        // 驗證 Ollama 顯示切換
        expect(document.getElementById('ollama-url-group').style.display).toBe('block');
    });

    it('saveConfig 應該從 DOM 讀取參數並對後端發起儲存 invoke', async () => {
        // 設定 DOM 被使用者異動後的狀態
        document.getElementById('api-provider').value = 'Gemini';
        document.getElementById('api-key').value = 'new-gemini-key';
        document.getElementById('input-path').value = '/new/input';
        document.getElementById('batch-size').value = '50';
        document.getElementById('chk-skip-json').checked = false;
        document.getElementById('chk-skip-jar').checked = true;

        await configModule.saveConfig();

        // 驗證 invoke 呼叫次數與內容
        expect(mockInvoke).toHaveBeenCalledWith('save_api_key_cmd', { key: 'new-gemini-key' });
        expect(mockInvoke).toHaveBeenCalledWith('save_config', expect.objectContaining({
            config: expect.objectContaining({
                api_provider: 'Gemini',
                path: '/new/input',
                batch_size: 50,
                skip_json: false,
                skip_jar: true
            })
        }));
    });

    it('loadModels 應該獲取模型清單並動態渲染 <option>', async () => {
        mockInvoke.mockResolvedValue(['gemini-1.5-pro', 'gemini-1.5-flash']);
        stateModule.state.currentLabels = { prompt_select_model: '請選取模型' };

        document.getElementById('api-provider').value = 'Gemini';

        await configModule.loadModels();

        const selectModel = document.getElementById('selected-model');
        expect(selectModel.options.length).toBe(3); // 1預設 + 2模型
        expect(selectModel.options[1].value).toBe('gemini-1.5-pro');
        expect(selectModel.options[2].value).toBe('gemini-1.5-flash');
    });
});
