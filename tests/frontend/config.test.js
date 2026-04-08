import { describe, it, expect, beforeEach, beforeAll, vi } from 'vitest';

// Mock 外部模組防止交互副作用 (提升至頂層)
vi.mock('../../frontend/modules/utils.js', () => ({
    appendLog: vi.fn(),
}));
vi.mock('../../frontend/modules/i18n.js', () => ({
    updateUiLanguage: vi.fn(),
}));

describe('config.js 設定管理模組', () => {
    let mockInvoke;
    let configModule;
    let stateModule;

    beforeAll(async () => {
        // 1. Mock Tauri API
        mockInvoke = vi.fn();
        globalThis.window = {
            __TAURI__: {
                core: { invoke: mockInvoke },
            },
        };

        // 2. 動態載入
        configModule = await import('../../frontend/modules/config.js');
        stateModule = await import('../../frontend/modules/state.js');
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
            <div id="api-base-url-group" style="display:block">
                <input id="api-base-url" />
            </div>
            <button id="btn-translate"></button>
        `;

        // 重設 Mock
        mockInvoke.mockReset();
        mockInvoke.mockImplementation(async (cmd, args) => {
            if (cmd === 'derive_config_ui_state_cmd') {
                const provider = args?.provider || '無';
                const noKeyProviders = ['Ollama', 'Google Free', '無'];
                const hideKey = noKeyProviders.includes(provider);
                return {
                    show_ollama_url: provider === 'Ollama',
                    show_api_key: !hideKey,
                    show_api_base_url: !hideKey,
                    show_fast_convert: false,
                    can_translate: true,
                };
            }
            if (cmd === 'normalize_form_config_cmd') return args?.config || {};
            return null;
        });
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
            api_base_url: 'https://api.custom-endpoint.com',
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
            model: 'llama3',
        };

        mockInvoke.mockImplementation(async (cmd) => {
            if (cmd === 'get_config') return fakeConfig;
            if (cmd === 'get_api_key_cmd') return 'test-key-123';
            if (cmd === 'get_models_from_provider') return ['llama3', 'mistral'];
            if (cmd === 'derive_config_ui_state_cmd') {
                return {
                    show_ollama_url: true,
                    show_api_key: false,
                    show_api_base_url: false,
                    show_fast_convert: false,
                    can_translate: true,
                };
            }
            if (cmd === 'normalize_form_config_cmd') return args?.config || {};
            return null;
        });

        await configModule.loadConfig();

        expect(document.getElementById('api-provider').value).toBe('Ollama');
        expect(document.getElementById('api-key').value).toBe('test-key-123');
        expect(document.getElementById('api-base-url').value).toBe('https://api.custom-endpoint.com');
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
        expect(mockInvoke).toHaveBeenCalledWith(
            'save_config',
            expect.objectContaining({
                config: expect.objectContaining({
                    api_provider: 'Gemini',
                    path: '/new/input',
                    batch_size: 50,
                    skip_json: false,
                    skip_jar: true,
                }),
            })
        );
    });

    it('loadModels 應該獲取模型清單並動態渲染 <option>', async () => {
        mockInvoke.mockResolvedValue(['gemini-1.5-pro', 'gemini-1.5-flash']);
        stateModule.state.currentLabels = { prompt_select_model: '請選取模型' };

        document.getElementById('api-provider').value = 'Gemini';
        document.getElementById('api-base-url').value = '';

        await configModule.loadModels();

        const selectModel = document.getElementById('selected-model');
        expect(selectModel.options.length).toBe(3);
        expect(selectModel.options[1].value).toBe('gemini-1.5-pro');
        expect(selectModel.options[2].value).toBe('gemini-1.5-flash');
    });

    describe('異常處理 (Catch Blocks)', () => {
        it('loadConfig 失敗時應該擷取異常並觸發 appendLog', async () => {
            mockInvoke.mockImplementation(async (cmd) => {
                if (cmd === 'get_config') throw 'backend-error';
                return null;
            });
            const { appendLog } = await import('../../frontend/modules/utils.js');
            stateModule.state.currentLabels = { status_load_config_failed: '載入失敗: {}' };

            await configModule.loadConfig();

            expect(appendLog).toHaveBeenCalledWith(expect.stringContaining('backend-error'));
        });

        it('saveConfig 失敗時應該擷取異常並觸發 appendLog', async () => {
            mockInvoke.mockImplementation(async (cmd) => {
                if (cmd === 'save_api_key_cmd') throw 'save-key-failed';
                return null;
            });
            const { appendLog } = await import('../../frontend/modules/utils.js');
            stateModule.state.currentLabels = { status_save_config_failed: '儲存失敗: {}' };

            await configModule.saveConfig();

            expect(appendLog).toHaveBeenCalledWith(expect.stringContaining('save-key-failed'));
        });

        it('loadModels 失敗時應該捕獲異常並顯示 label_no_models', async () => {
            mockInvoke.mockImplementation(async (cmd) => {
                if (cmd === 'get_models_from_provider') throw 'fetch-models-failed';
                return null;
            });
            stateModule.state.currentLabels = { label_no_models: '無可用模型' };
            document.getElementById('api-provider').value = 'Gemini';
            document.getElementById('api-base-url').value = '';

            await configModule.loadModels();

            const selectModel = document.getElementById('selected-model');
            expect(selectModel.innerHTML).toContain('無可用模型');
        });

        it('loadModels 失敗時若錯誤訊息在 currentLabels 中應顯示具體錯誤', async () => {
            mockInvoke.mockImplementation(async (cmd) => {
                if (cmd === 'get_models_from_provider') throw 'err_api_key_empty';
                return null;
            });
            stateModule.state.currentLabels = {
                label_no_models: '無可用模型',
                err_api_key_empty: 'API Key 為空，請先填入 API Key',
            };
            document.getElementById('api-provider').value = 'Gemini';
            document.getElementById('api-base-url').value = '';

            await configModule.loadModels();

            const selectModel = document.getElementById('selected-model');
            expect(selectModel.innerHTML).toContain('API Key 為空，請先填入 API Key');
        });
    });

    describe('恢復預設完整性測試 (Dirty Check)', () => {
        const { readFileSync } = require('fs');
        const { resolve } = require('path');
        const mockDefaultConfig = {
            api_provider: '無',
            model: '',
            ollama_url: 'http://localhost:11434',
            api_base_url: '',
            batch_size: 150,
            batch_max_chars: 3500,
            timeout: 60,
            pack_format: 15,
            glossary_priority: 'official',
            system_prompt:
                '\n\n[內部技術指令 - 請務必遵守]\n1. 僅針對 %%VAR_n%%, %%MC_n%%, %%HEX_n%% 等技術佔位符執行「保持原樣」操作（不可修改、翻譯或增刪標籤）。\n2. 除上述佔位符外的其餘文本內容均「必須」按要求翻譯，絕對不可將全文原樣輸出。',
            user_prompt:
                '你是一位專業的 Minecraft 模組翻譯員。現在請將以下模組字串翻譯為「繁體中文 (zh_tw)」。\n保持專業的遊戲術語風格（如方塊、實體、附魔）。',
            excluded_paths: [
                'kubejs/data/',
                'packmenu/',
                'config/almostunified/',
                'fancymenu/',
                'journeymap/icon/theme',
                'shaderpacks/',
                'screenshots/',
                'saves/',
                'logs/',
                'defaultconfigs/',
                'local/',
                '.mixin.out/',
            ],
            skip_json: false,
            skip_js: false,
            skip_jar: false,
            skip_book: false,
            enable_llm_log: false,
            enable_debug_log: false,
            source_lang: 'en_us',
            target_lang: 'zh_tw',
            ui_lang: 'zh_tw',
            output_dir: '',
            show_api_settings: false,
            show_developer_mode: false,
            show_debug_tools: false,
            main_x: 50.0,
            main_y: 50.0,
            main_width: 800.0,
            main_height: 600.0,
            viewer_x: 100.0,
            viewer_y: 100.0,
            viewer_width: 800.0,
            viewer_height: 600.0,
            fast_convert: false,
        };

        beforeEach(() => {
            // 💡 關鍵：載入真實的 HTML，確保能偵測到未來新增的欄位
            const htmlPath = resolve(__dirname, '../../frontend/index.html');
            const html = readFileSync(htmlPath, 'utf-8');
            document.body.innerHTML = html;

            stateModule.state.currentConfig = { ...mockDefaultConfig };
            stateModule.state.currentLabels = { status_config_restored: 'OK', status_dev_restored: 'OK' };

            mockInvoke.mockImplementation(async (cmd) => {
                if (cmd === 'get_default_config') return mockDefaultConfig;
                return {};
            });

            // 防止 fetch 引發連線錯誤 (例如 Ollama 清單獲取)
            global.fetch = vi.fn(() =>
                Promise.resolve({
                    ok: true,
                    json: () => Promise.resolve([]),
                })
            );
        });

        it('API 面板重置：應能重置面板內所有元件，且不干涉開發者面板', async () => {
            const apiPanel = document.getElementById('api-settings');
            const devPanel = document.getElementById('developer-settings');

            const apiInputs = Array.from(apiPanel.querySelectorAll('input, select, textarea'));
            const devInputs = Array.from(devPanel.querySelectorAll('input, select, textarea'));

            // 1. 全部設為髒值 (Dirty)
            apiInputs.forEach((el) => {
                if (el.type === 'checkbox') el.checked = true;
                else el.value = 'DIRTY_API';
            });
            devInputs.forEach((el) => {
                if (el.type === 'checkbox') el.checked = true;
                else el.value = 'DIRTY_DEV';
            });

            // 2. 執行 API 重置
            await configModule.restoreDefaultConfig();

            // 3. 自動檢查 API 面板中的每一個元件是否都已「不再是髒值」
            const missingIds = [];
            apiInputs.forEach((el) => {
                if (el.id === 'api-key' || el.id === 'ui-lang') return;

                // 元件 ID 與 Config Key 的對應邏輯
                let configKey = el.id.replace('chk-', '').replace('sel-', '').replace(/-/g, '_');
                if (el.id === 'timeout-sec') configKey = 'timeout'; // 特殊映射範例

                if (el.type === 'checkbox') {
                    // 如果預設是 false，重置後應該是 false，不應該還是 true
                    if (el.checked === true && !mockDefaultConfig[configKey]) missingIds.push(el.id);
                } else if (el.value === 'DIRTY_API') {
                    missingIds.push(el.id);
                }
            });

            if (missingIds.length > 0) {
                throw new Error(`❌ 偵測到「API 面板」中有元件未被重置邏輯覆蓋：${missingIds.join(', ')}`);
            }

            // 4. 驗證排除清單（開發者面板）仍為髒值 (獨立性)
            expect(document.getElementById('excluded-paths').value).toBe('DIRTY_DEV');
        });

        it('開發人員面板重置：應能重置面板內所有元件，自動偵測漏設定的項目', async () => {
            const devPanel = document.getElementById('developer-settings');
            const devInputs = Array.from(devPanel.querySelectorAll('input, select, textarea'));

            // 1. 全部設為髒值
            devInputs.forEach((el) => {
                if (el.type === 'checkbox') el.checked = true;
                else el.value = 'DIRTY_DEV';
            });

            // 2. 執行開發者重置
            await configModule.restoreDevDefaults();

            // 3. 自動檢查有無漏網之魚
            const missingIds = [];
            devInputs.forEach((el) => {
                if (el.type === 'checkbox') {
                    let configKey = el.id.replace('chk-', '').replace(/-/g, '_');
                    if (el.checked === true && !mockDefaultConfig[configKey]) missingIds.push(el.id);
                } else if (el.value === 'DIRTY_DEV') {
                    missingIds.push(el.id);
                }
            });

            if (missingIds.length > 0) {
                throw new Error(`❌ 偵測到「開發人員面板」中有元件未被重置邏輯覆蓋：${missingIds.join(', ')}`);
            }
        });
    });

    describe('loadTranslationLangs 異常處理', () => {
        it('invoke 失敗時應該捕獲異常並輸出錯誤', async () => {
            document.body.innerHTML += `
                <select id="source-lang"></select>
                <select id="target-lang"></select>
            `;
            mockInvoke.mockImplementation(async (cmd) => {
                if (cmd === 'get_available_translation_langs') throw 'lang-fetch-failed';
                return null;
            });
            const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

            await configModule.loadTranslationLangs();

            expect(consoleErrorSpy).toHaveBeenCalled();
            consoleErrorSpy.mockRestore();
        });
    });

    describe('loadModels 非陣列回傳處理', () => {
        it('當 models 不是陣列時應該顯示 label_no_models', async () => {
            mockInvoke.mockResolvedValue('not-an-array');
            stateModule.state.currentLabels = { label_no_models: '無可用模型', prompt_select_model: '請選取模型' };
            document.getElementById('api-provider').value = 'Gemini';
            document.getElementById('api-base-url').value = '';

            await configModule.loadModels();

            const selectModel = document.getElementById('selected-model');
            expect(selectModel.innerHTML).toContain('無可用模型');
        });
    });
});
