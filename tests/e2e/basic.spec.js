import { test, expect } from '@playwright/test';

const TAURI_MOCK = () => {
    window.__TAURI__ = {
        core: {
            invoke: async (cmd, args) => {
                const defaultConfig = {
                    api_provider: 'Gemini',
                    path: '',
                    ollama_url: 'http://localhost:11434',
                    api_base_url: '',
                    batch_size: 150,
                    batch_max_chars: 3500,
                    timeout: 60,
                    pack_format: 15,
                    glossary_priority: 'official',
                    ui_lang: 'zh_tw',
                    output_dir: '',
                    system_prompt: '',
                    user_prompt: '',
                    skip_json: false,
                    skip_js: false,
                    skip_jar: false,
                    skip_book: false,
                    enable_llm_log: false,
                    model: '',
                    show_api_settings: false,
                    show_developer_mode: false,
                    show_debug_tools: false,
                    main_x: 50,
                    main_y: 50,
                    main_width: 800,
                    main_height: 600,
                    viewer_x: 100,
                    viewer_y: 100,
                    viewer_width: 800,
                    viewer_height: 600,
                    fast_convert: false,
                    source_lang: 'en_us',
                    target_lang: 'zh_tw',
                };
                const defaultStyle = {
                    theme: 'dark',
                    dark_bg: [30, 30, 35],
                    dark_text: [255, 255, 255],
                    dark_btn_bg: [0, 100, 200],
                    dark_input_bg: [10, 20, 30],
                    dark_list_bg: [40, 50, 60],
                    dark_tab_active: [70, 80, 90],
                    dark_tab_inactive: [100, 110, 120],
                    dark_label: [130, 140, 150],
                    font_size: 16,
                    btn_rounding_enabled: true,
                    btn_rounding_value: 5,
                    progress_pulse_enabled: false,
                    progress_pulse_speed: 2,
                    show_palette_settings: false,
                    instance_overrides: {},
                };

                if (cmd === 'get_gui_init_state') {
                    return {
                        config: defaultConfig,
                        style: defaultStyle,
                        labels: {},
                        css_vars: {},
                        ui_patch: {
                            status: 'IDLE',
                            show_translate: true,
                            show_pause: false,
                            show_resume: false,
                            show_stop: false,
                            lock_controls: false,
                            pause_notice: '',
                            clear_current_status: true,
                            clear_batch_status: true,
                        },
                        toggle_labels: {},
                    };
                }
                if (cmd === 'get_config') {
                    return defaultConfig;
                }
                if (cmd === 'get_api_key_cmd') return '';
                if (cmd === 'get_models_from_provider') return [];
                if (cmd === 'get_i18n_labels') {
                    return {
                        btn_run: '翻譯',
                        btn_select_file: '選擇檔案',
                        btn_select_folder: '選擇資料夾',
                        btn_output_dir: '輸出路徑',
                        btn_open_output: '打開輸出',
                        btn_save_config: '儲存設定',
                        btn_nav_settings: 'API & 設定',
                        btn_nav_dict: '建議詞管理',
                        btn_nav_palette: '主題 & 調色盤',
                        btn_nav_theme: '切換主題',
                        btn_nav_dev: '開發者選項',
                        label_input_path: '輸入路徑',
                        label_output_path: '輸出路徑',
                        label_provider: '提供者',
                        label_model: '模型',
                        label_max_chars: '最大字元數',
                        label_timeout: '逾時 (秒)',
                        label_glossary_priority: '辭典優先級',
                        prompt_select_model: '請選取模型',
                        placeholder_input_path: '輸入路徑...',
                        placeholder_output_path: './LLMTranslator (預設)',
                        placeholder_search_terms: '搜尋字典...',
                        placeholder_dict_key: '原文...',
                        placeholder_dict_value: '翻譯...',
                        header_api_settings: 'API 設定',
                        header_dict_mgr: '字典管理',
                        glossary_priority_user: '使用者優先',
                        glossary_priority_official: '官方優先',
                        label_enable_log: '啟用日誌',
                        label_disable_log: '停用日誌',
                        label_skip_json: '跳過 JSON',
                        label_no_skip_json: '處理 JSON',
                        label_skip_js: '跳過 JS',
                        label_no_skip_js: '處理 JS',
                        label_skip_jar: '跳過 JAR',
                        label_no_skip_jar: '處理 JAR',
                        label_skip_book: '跳過 Book',
                        label_no_skip_book: '處理 Book',
                        label_page_info: '第 {} / {} 頁',
                        glossary_key: '原文:',
                        glossary_value: '翻譯:',
                        glossary_clear_title: '確定清除?',
                        status_dict_load_failed: '讀取失敗 {}',
                        status_dict_add_failed: '新增失敗 {}',
                        status_dict_item_delete_confirm: '刪除 {}?',
                        status_dict_key_empty: 'Key 為空',
                        status_dict_replace_confirm: '替換 {} -> {}?',
                        status_dict_replace_empty: '替換不可為空',
                        status_dict_replace_failed: '替換失敗 {}',
                        status_dict_add_success: '新增 {} -> {}',
                        status_dict_item_updated: '更新 {}',
                        status_dict_clear_success: '已清除',
                        status_dict_export_success: '已導出 {}',
                        status_open_path_failed: '打開失敗 {}',
                        status_load_config_failed: '載入失敗: {}',
                        status_save_config_failed: '儲存失敗: {}',
                        status_config_restored: 'API 設定已重置',
                        status_dev_restored: '開發者設定已重置',
                        status_browse_path_failed: '瀏覽路徑失敗: {}',
                        status_palette_clear_item: '已清除 {} 覆寫',
                        label_bg_color: '背景色',
                        label_text_color: '文字色',
                        label_palette_color: '顏色',
                        label_palette_target_type: '目標類型',
                        label_palette_target_item: '目標項目',
                        label_palette_property: '屬性',
                        label_palette_rounding: '圓角',
                        label_user_prompt: '使用者提示',
                        label_system_prompt: '系統提示',
                        default_user_prompt: '',
                        default_system_prompt: '',
                        label_no_models: '無可用模型',
                        btn_clear: '清除',
                        btn_translate: '翻譯',
                        btn_save: '儲存',
                        btn_add: '新增',
                        btn_replace: '替換',
                        btn_import: '匯入',
                        btn_export: '匯出',
                        btn_page_prev: '上一頁',
                        btn_page_next: '下一頁',
                        btn_restore_api: '重置 API',
                        btn_restore_dev: '重置開發者',
                        btn_restore_palette: '重置調色盤',
                        tab_user: '使用者',
                        tab_official: '官方',
                        progress_style_solid: '實心',
                        progress_style_gradient: '漸層',
                        progress_style_striped: '條紋',
                        label_progress_pulse: '脈動效果',
                        label_pulse_speed: '脈動速度',
                        label_font_size: '字型大小',
                        label_btn_rounding: '按鈕圓角',
                        label_palette_number: '數值',
                        label_palette_clear_item: '清除覆寫',
                        label_palette_clear_all: '清除全部',
                        label_open_json: '打開 JSON',
                        label_debug_rendered: '渲染數',
                        label_debug_scroll: '滾動鎖定',
                        label_debug_total: '總計',
                        label_debug_memory: '記憶體',
                        label_fast_convert: '簡繁轉換',
                        spec_area_dict: '字典區域',
                    };
                }
                if (cmd === 'get_style_config') {
                    return defaultStyle;
                }
                if (cmd === 'derive_panel_state_cmd') {
                    const next = { ...(args.current || {}) };
                    if (args.action === 'toggle_api') {
                        next.show_api_settings = !next.show_api_settings;
                        if (next.show_api_settings) {
                            next.show_developer_mode = false;
                            next.show_palette_settings = false;
                        }
                    }
                    if (args.action === 'toggle_dev') {
                        next.show_developer_mode = !next.show_developer_mode;
                        if (next.show_developer_mode) {
                            next.show_api_settings = false;
                            next.show_palette_settings = false;
                        }
                    }
                    if (args.action === 'toggle_palette') {
                        next.show_palette_settings = !next.show_palette_settings;
                        if (next.show_palette_settings) {
                            next.show_api_settings = false;
                            next.show_developer_mode = false;
                        }
                    }
                    return next;
                }
                if (cmd === 'toggle_theme_style_cmd') {
                    return { ...(args.style || defaultStyle), theme: args.style?.theme === 'dark' ? 'light' : 'dark' };
                }
                if (cmd === 'get_dictionary_page') return { items: [], total_pages: 1, page: 0, page_size: 10 };
                if (cmd === 'get_available_translation_langs') return ['zh_tw', 'zh_cn', 'ja_jp'];
                if (cmd === 'get_available_langs') return ['zh_tw', 'zh_cn', 'ja_jp', 'en_us'];
                if (cmd === 'derive_toggle_labels_cmd') return {};
                if (cmd === 'derive_config_ui_state_cmd')
                    return {
                        show_ollama_url: false,
                        show_api_key: true,
                        show_api_base_url: true,
                        show_fast_convert: true,
                        can_translate: true,
                    };
                if (cmd === 'derive_palette_state_cmd')
                    return {
                        show_clear_group: false,
                        show_property_group: false,
                        show_color_group: true,
                        show_number_group: false,
                        label_palette_number: '數值',
                        label_palette_color: '顏色',
                        number_value: 0,
                        number_step: 1,
                        color_value: '#000000',
                    };
                if (cmd === 'get_gui_css_vars') return {};
                if (cmd === 'build_form_config_cmd') return { ...(args.base || {}), ...(args.input || {}) };
                if (cmd === 'build_style_from_form_cmd') return { ...(args.base || defaultStyle) };
                if (cmd === 'apply_palette_mutation_cmd') return { ...(args.style || defaultStyle) };
                if (cmd === 'clear_palette_override_cmd') return { ...(args.style || defaultStyle) };
                if (cmd === 'save_config') return {};
                if (cmd === 'save_style_config') return {};
                if (cmd === 'show_window') return {};
                if (cmd === 'open_dict_window') return {};
                if (cmd === 'setup_dev_mock') return true;
                if (cmd === 'derive_default_prompts') return { default_user_prompt: '', default_system_prompt: '' };
                return null;
            },
        },
        event: {
            listen: async () => () => {},
            emit: async () => {},
        },
    };
};

test.beforeEach(async ({ page }) => {
    await page.addInitScript(TAURI_MOCK);
    await page.goto('/');
    await page.waitForFunction(() => window.__logViewer !== undefined, { timeout: 10000 });
});

test('主頁面應正確載入並顯示主要元件', async ({ page }) => {
    await expect(page.locator('#btn-translate')).toBeVisible();
    await expect(page.locator('#input-path')).toBeVisible();
    await expect(page.locator('#output-dir')).toBeVisible();
    await expect(page.locator('#api-provider')).toBeVisible();
    await expect(page.locator('#log-output')).toBeVisible();
});

test('設定面板應可展開並顯示配置', async ({ page }) => {
    await page.locator('#btn-nav-api').click();
    await expect(page.locator('.api-settings')).toHaveClass(/expanded/);
    await expect(page.locator('#api-provider')).toBeVisible();
    await expect(page.locator('#selected-model')).toBeVisible();
    await expect(page.locator('#batch-size')).toBeVisible();
});

test('字典按鈕應觸發開啟字典視窗', async ({ page }) => {
    const invokeCalls = [];
    await page.exposeFunction('trackInvoke', (cmd) => {
        invokeCalls.push(cmd);
    });
    await page.addInitScript(() => {
        const origInvoke = window.__TAURI__.core.invoke;
        window.__TAURI__.core.invoke = async (cmd, args) => {
            window.trackInvoke(cmd);
            return origInvoke(cmd, args);
        };
    });
    await page.goto('/');
    await page.waitForFunction(() => window.__logViewer !== undefined, { timeout: 10000 });

    await page.locator('#btn-nav-dict').click();
    expect(invokeCalls).toContain('open_dict_window');
});

test('主題切換按鈕應可點擊並觸發切換', async ({ page }) => {
    const invokeCalls = [];
    await page.exposeFunction('trackInvoke', (cmd) => {
        invokeCalls.push(cmd);
    });
    await page.addInitScript(() => {
        const origInvoke = window.__TAURI__.core.invoke;
        window.__TAURI__.core.invoke = async (cmd, args) => {
            window.trackInvoke(cmd);
            return origInvoke(cmd, args);
        };
    });
    await page.goto('/');
    await page.waitForFunction(() => window.__logViewer !== undefined, { timeout: 10000 });

    await page.locator('#btn-nav-theme').click();
    expect(invokeCalls).toContain('save_style_config');
});

test('開發者面板應可展開', async ({ page }) => {
    await page.locator('#btn-nav-dev').click();
    await page.waitForTimeout(500);
    await expect(page.locator('#excluded-paths')).toBeVisible();
});

test('開發者面板切換開關時不應影響其他開關，也不應觸發整頁語系刷新', async ({ page }) => {
    await page.locator('#btn-nav-dev').click();
    await expect(page.locator('#chk-skip-json')).toBeVisible();

    const i18nCallsBefore = await page.evaluate(() => {
        const invoke = window.__TAURI__.core.invoke;
        let count = 0;
        window.__TAURI__.core.invoke = async (cmd, args) => {
            if (cmd === 'get_i18n_labels') count += 1;
            return invoke(cmd, args);
        };
        window.__getI18nCallCount = () => count;
        return window.__getI18nCallCount();
    });
    expect(i18nCallsBefore).toBe(0);

    await page.evaluate(() => {
        const setToggle = (id, value) => {
            const el = document.getElementById(id);
            if (!el) return;
            el.checked = value;
            el.dispatchEvent(new Event('change', { bubbles: true }));
        };
        setToggle('chk-skip-json', true);
        setToggle('chk-debug-log', true);
    });
    await page.waitForTimeout(200);

    await expect(page.locator('#chk-skip-json')).toBeChecked();
    await expect(page.locator('#chk-debug-log')).toBeChecked();
    await expect(page.locator('#chk-skip-js')).not.toBeChecked();
    await expect(page.locator('#chk-skip-jar')).not.toBeChecked();
    await expect(page.locator('#chk-skip-book')).not.toBeChecked();
    await expect(page.locator('#chk-llm-log')).not.toBeChecked();

    const i18nCallsAfter = await page.evaluate(() => window.__getI18nCallCount());
    expect(i18nCallsAfter).toBe(0);
});

test('開發者面板開關列應為左開關右文字', async ({ page }) => {
    await page.locator('#btn-nav-dev').click();
    await expect(page.locator('#chk-skip-json')).toBeVisible();

    const isSwitchFirst = await page.evaluate(() => {
        const group = document.querySelector('#developer-settings .switch-group');
        if (!group) return false;
        const first = group.firstElementChild;
        const second = group.children[1];
        if (!first || !second) return false;
        const firstIsSwitch = first.classList.contains('switch');
        const secondIsText = second.id?.startsWith('label-');
        return firstIsSwitch && secondIsText;
    });
    expect(isSwitchFirst).toBe(true);
});
