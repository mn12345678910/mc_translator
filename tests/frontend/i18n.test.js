import { describe, it, expect, beforeEach, beforeAll, vi } from 'vitest';

describe('i18n.js 介面語言模組', () => {
    let mockInvoke;
    let i18nModule;
    let stateModule;

    beforeAll(async () => {
        // 1. 在載入檔案前 Mock Tauri API
        mockInvoke = vi.fn();
        globalThis.window = {
            __TAURI__: {
                core: { invoke: mockInvoke }
            }
        };

        // 2. 動態載入相關模組
        i18nModule = await import('../../frontend/modules/i18n.js');
        stateModule = await import('../../frontend/modules/state.js');
    });

    beforeEach(() => {
        // 重設 DOM 環境
        document.body.innerHTML = `
            <select id="ui-lang"></select>
            <textarea id="user-prompt"></textarea>
            <select id="selected-model">
                <option value="">請選取模型</option>
                <option value="gpt-4">GPT 4</option>
            </select>
            <button id="btn-translate"></button>
        `;

        // 重設 Mock 控制器
        mockInvoke.mockReset();
        stateModule.state.currentLabels = {}; // 清空 State
    });

    it('loadUiLangs 應該填入所有基本語言選單', async () => {
        mockInvoke.mockResolvedValue(['en_us']); // 後端僅支援 en_us

        await i18nModule.loadUiLangs();

        const select = document.getElementById('ui-lang');
        expect(select.options.length).toBe(4); // 應該包含 zh_tw, zh_cn, en_us, ja_jp 聯集
        expect(select.querySelector('option[value="zh_cn"]')).toBeTruthy();
    });

    it('updateUiLanguage 應該依據 Labels 更新模型選單 Placeholder', async () => {
        // 模擬切換到英文的情境
        const mockLabels = {
            prompt_select_model: 'Select a Model (EN)', // 英文
            btn_run_trans: 'Start',
        };
        mockInvoke.mockResolvedValue(mockLabels);

        // 模擬「舊標籤」狀態為繁體
        stateModule.state.currentLabels = {
            prompt_select_model: '請選取模型', // 繁體
        };

        const selectModel = document.getElementById('selected-model');
        const firstOpt = selectModel.options[0];
        firstOpt.textContent = '請選取模型'; // 畫面當前的文字（舊語言）

        await i18nModule.updateUiLanguage();

        // 驗證 Placeholder 變成英文
        expect(firstOpt.textContent).toBe('Select a Model (EN)');
    });

    it('updateUiLanguage 不應該在 User Prompt 符合預設簡體提示字串時更新它', async () => {
        const mockLabels = {
            default_user_prompt: 'English prompt defaults...',
        };
        mockInvoke.mockResolvedValue(mockLabels);

        const promptArea = document.getElementById('user-prompt');
        // 加入含有「风格」的簡體中文提示字串（已被我們修好的狀態）
        const original = '你是一位专业的 Minecraft 模组翻译员。现在请将以下模组字串翻译为「简体中文 (zh_cn)」。\n保持专业的游戏术语风格（如方块、实体、附魔）。';
        promptArea.value = original;

        await i18nModule.updateUiLanguage();

        // UI 語系切換不再改動使用者提示
        expect(promptArea.value.trim()).toBe(original.trim());
    });
    it('loadUiLangs 應該在 invoke 拋出異常時妥善處理', async () => {
        const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
        mockInvoke.mockRejectedValue(new Error('Backend Error'));

        await i18nModule.loadUiLangs();

        expect(consoleErrorSpy).toHaveBeenCalled();
        consoleErrorSpy.mockRestore();
    });

    it('updateUiLanguage 應該在 labels 缺失時支援 Fallback 備援', async () => {
        const mockLabels = {
            app_title: 'MCTitle',
            label_provider: 'Provider',
        };
        mockInvoke.mockResolvedValue(mockLabels);

        const labelEl = document.createElement('label');
        labelEl.setAttribute('for', 'api-provider');
        labelEl.setAttribute('data-i18n', 'label_provider');
        document.body.appendChild(labelEl);

        await i18nModule.updateUiLanguage();

        expect(labelEl.textContent).toBe('Provider');
    });

    it('updateToggleStateLabel 應該正確更新所有開關文字', () => {
        stateModule.state.currentLabels = {
            glossary_priority_user: 'User First',
            glossary_priority_official: 'Official First',
            label_enable_log: 'Log On',
            label_disable_log: 'Log Off',
        };

        document.body.innerHTML += `
            <div id="label-glossary-priority"></div>
            <div id="label-llm-log"></div>
        `;

        i18nModule.updateToggleStateLabel('chk-glossary-priority', true);
        expect(document.getElementById('label-glossary-priority').textContent).toBe('User First');

        i18nModule.updateToggleStateLabel('chk-glossary-priority', false);
        expect(document.getElementById('label-glossary-priority').textContent).toBe('Official First');

        i18nModule.updateToggleStateLabel('chk-llm-log', true);
        expect(document.getElementById('label-llm-log').textContent).toBe('Log On');
    });
    it('updateUiLanguage 應該更新輸入框的 Placeholder', async () => {
        const mockLabels = {
            placeholder_search_terms: '搜尋字典...',
        };
        mockInvoke.mockResolvedValue(mockLabels);

        document.body.innerHTML += `
            <input id="dict-search" placeholder="Original" />
        `;

        await i18nModule.updateUiLanguage();

        expect(document.getElementById('dict-search').placeholder).toBe('搜尋字典...');
    });

    it('updateUiLanguage 應該覆蓋完整的 DOM 視覺映射與屬性替換', async () => {
        const mockLabels = {
            btn_select_file: 'Select File',
            btn_select_folder: 'Select Folder',
            btn_output_dir: 'Output Dir',
            btn_open_output: 'Open Output',
            btn_run_trans: 'Translate',
            btn_save_config: 'Save Config',
            header_api_settings: 'API Settings',
            label_provider: 'Provider',
            label_model: 'Model',
            label_max_chars: 'Max Chars',
            label_timeout: 'Timeout',
            label_glossary_priority: 'Priority',
            label_palette_target_type: 'Target Type',
            label_palette_target_item: 'Target Item',
            label_palette_property: 'Property',
            label_palette_color: 'Color',
            label_palette_rounding: 'Rounding',
            label_user_prompt: 'User Prompt',
            label_system_prompt: 'System Prompt',
            label_input_path: 'Input Path',
            label_output_path: 'Output Path',
            placeholder_search_terms: 'Search...',
            placeholder_dict_key: 'Key...',
            placeholder_dict_value: 'Value...',
            placeholder_input_path: 'Input Path...',
            header_dict_mgr: 'Mgr',
            glossary_priority_hover: 'Hover',
            title_some_key: 'Tooltip Title'
        };

        mockInvoke.mockResolvedValue(mockLabels);

        document.body.innerHTML += `
            <!-- option mapping selects -->
            <select id="palette-target-type">
                <option value="global"></option>
                <option value="specific"></option>
            </select>
            <select id="api-provider">
                <option value="Ollama"></option>
                <option value="無"></option>
            </select>
            <select id="palette-target-item">
                <option value="dark_bg"></option>
            </select>
            <select id="palette-property">
                <option value="bg"></option>
            </select>
            <div id="header-dict-mgr"></div>
            <div>
                 <input id="chk-glossary-priority" />
            </div>
            <div data-i18n-title="title_some_key"></div>

            <!-- mapping -->


            <button id="btn-browse-file" data-i18n="btn_select_file"></button>
            <button id="btn-browse-dir" data-i18n="btn_select_folder"></button>
            <button id="btn-browse-output" data-i18n="btn_output_dir"></button>
            <button id="btn-browse-output-open" data-i18n="btn_open_output"></button>
            <button id="btn-translate" data-i18n="btn_run_trans"></button>
            <button id="btn-save-config" data-i18n="btn_save_config"></button>
            <button id="header-api-settings" data-i18n="header_api_settings"></button>

            <!-- label[for] -->
            <label for="api-provider" data-i18n="label_provider"></label>
            <label for="selected-model" data-i18n="label_model"></label>
            <label for="batch-max-chars" data-i18n="label_max_chars"></label>
            <label for="timeout-sec" data-i18n="label_timeout"></label>
            <label for="glossary-priority" data-i18n="label_glossary_priority"></label>
            <label for="palette-target-type" data-i18n="label_palette_target_type"></label>
            <label for="palette-target-item" data-i18n="label_palette_target_item"></label>
            <label for="palette-property" data-i18n="label_palette_property"></label>
            <label for="palette-color" data-i18n="label_palette_color"></label>
            <label for="palette-rounding" data-i18n="label_palette_rounding"></label>
            <label for="user-prompt" data-i18n="label_user_prompt"></label>
            <label for="system-prompt" data-i18n="label_system_prompt"></label>
            <label for="input-path" data-i18n="label_input_path"></label>
            <label for="output-dir" data-i18n="label_output_path"></label>

            <!-- placeholder -->
            <input id="dict-search" placeholder="Def" data-i18n-placeholder="placeholder_search_terms" />
            <input id="dict-input-key" placeholder="Def" data-i18n-placeholder="placeholder_dict_key" />
            <input id="dict-input-value" placeholder="Def" data-i18n-placeholder="placeholder_dict_value" />
            <input id="input-path" placeholder="Def" data-i18n-placeholder="placeholder_input_path" />

        `;

        await i18nModule.updateUiLanguage();

        expect(document.getElementById('btn-browse-file').textContent).toBe('Select File');
        expect(document.querySelector('label[for="api-provider"]').textContent).toBe('Provider');
        expect(document.getElementById('dict-search').placeholder).toBe('Search...');
    });

    describe('額外涵蓋範圍 (Coverage Extension)', () => {
        it('updateUiLanguage 不應該在 System Prompt 符合預設提示時更新它', async () => {
            const mockLabels = { default_system_prompt: 'English system defaults...' };
            mockInvoke.mockResolvedValue(mockLabels);

            document.body.innerHTML += `<textarea id="system-prompt"></textarea>`;
            const promptArea = document.getElementById('system-prompt');
            const original = '\n\n[內部技術指令 - 請務必遵守]\n1. 僅針對 %%VAR_n%%, %%MC_n%%, %%HEX_n%% 等技術佔位符執行「保持原樣」操作（不可修改、翻譯或增刪標籤）。\n2. 除上述佔位符外的其餘文本內容均「必須」按要求翻譯，絕對不可將全文原樣輸出。';
            promptArea.value = original;

            await i18nModule.updateUiLanguage();

            expect(promptArea.value.trim()).toBe(original.trim());
        });

        it('updateUiLanguage 應該在 invoke 拋出異常時妥善處理 (Catch)', async () => {
            const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
            mockInvoke.mockRejectedValue(new Error('Labels Fetch Failed'));

            await i18nModule.updateUiLanguage();

            expect(consoleErrorSpy).toHaveBeenCalled();
            consoleErrorSpy.mockRestore();
        });

        it('updateToggleStateLabel 應該正確更新剩餘的跳過開關文字', () => {
            stateModule.state.currentLabels = {
                label_skip_json: 'Skip JSON ON',
                label_no_skip_json: 'Skip JSON OFF',
                label_skip_js: 'Skip JS ON',
                label_no_skip_js: 'Skip JS OFF',
                label_skip_jar: 'Skip JAR ON',
                label_no_skip_jar: 'Skip JAR OFF',
                label_skip_book: 'Skip Book ON',
                label_no_skip_book: 'Skip Book OFF',
            };

            const ids = ['chk-skip-json', 'chk-skip-js', 'chk-skip-jar', 'chk-skip-book'];
            ids.forEach(id => {
                 document.body.innerHTML += `<div id="label-${id.replace('chk-', '')}"></div>`;
            });

            i18nModule.updateToggleStateLabel('chk-skip-json', true);
            expect(document.getElementById('label-skip-json').textContent).toBe('Skip JSON ON');

            i18nModule.updateToggleStateLabel('chk-skip-json', false);
            expect(document.getElementById('label-skip-json').textContent).toBe('Skip JSON OFF');

            i18nModule.updateToggleStateLabel('chk-skip-js', true);
            expect(document.getElementById('label-skip-js').textContent).toBe('Skip JS ON');

            i18nModule.updateToggleStateLabel('chk-skip-jar', true);
            expect(document.getElementById('label-skip-jar').textContent).toBe('Skip JAR ON');

            i18nModule.updateToggleStateLabel('chk-skip-book', true);
            expect(document.getElementById('label-skip-book').textContent).toBe('Skip Book ON');
        });
    });
});
