// frontend/modules/i18n.js
import { state } from './state.js';
const { invoke } = window.__TAURI__ ? window.__TAURI__.core : { invoke: () => {} };

export async function loadUiLangs() {
    const uiLang = document.getElementById('ui-lang');
    if (!uiLang) return;
    try {
        const langs = await invoke('get_available_langs');
        uiLang.innerHTML = '';
        const allLangs = Array.from(new Set([...langs, 'zh_tw', 'zh_cn', 'en_us', 'ja_jp']));
        allLangs.forEach((l) => {
            const opt = document.createElement('option');
            opt.value = l;
            opt.textContent =
                l === 'zh_tw'
                    ? state.currentLabels.lang_zh_tw || '繁體中文 (zh_tw)'
                    : l === 'en_us'
                      ? state.currentLabels.lang_en_us || 'English (en_us)'
                      : l === 'zh_cn'
                        ? state.currentLabels.lang_zh_cn || '简体中文 (zh_cn)'
                        : l === 'ja_jp'
                          ? state.currentLabels.lang_ja_jp || '日本語 (ja_jp)'
                          : l;
            uiLang.appendChild(opt);
        });
    } catch (e) {
        console.error('無法載入語言清單', e);
    }
}

export async function updateUiLanguage() {
    const uiLang = document.getElementById('ui-lang');
    const btnNavApi = document.getElementById('btn-nav-api');
    const btnNavDict = document.getElementById('btn-nav-dict');
    const btnNavPalette = document.getElementById('btn-nav-palette');
    const btnNavTheme = document.getElementById('btn-nav-theme');
    const btnNavDev = document.getElementById('btn-nav-dev');
    const userPrompt = document.getElementById('user-prompt');
    const systemPrompt = document.getElementById('system-prompt');

    try {
        const labels = await invoke('get_i18n_labels', { lang: uiLang ? uiLang.value : undefined });
        if (!labels) return;
        const oldLabels = { ...state.currentLabels };
        state.currentLabels = { ...labels };

        if (uiLang && uiLang.value) {
            document.documentElement.lang = uiLang.value.replace('_', '-');
        }

        const titleNode = document.querySelector('h1 span') || document.querySelector('h1');
        if (titleNode && labels.app_title) titleNode.textContent = labels.app_title;

        const mapping = {
            'btn-browse-file': labels.btn_select_file,
            'btn-browse-dir': labels.btn_select_folder,
            'btn-browse-output': labels.btn_output_dir,
            'btn-browse-output-open': labels.btn_open_output,
            'btn-translate': labels.btn_run_trans,
            'btn-pause': labels.btn_pause,
            'btn-stop': labels.btn_stop,
            'btn-resume': labels.btn_resume,
            'btn-save-config': labels.btn_save_config,
            'btn-restore-config': labels.btn_restore_defaults,
            'btn-save-style': labels.btn_save_style,
            'btn-restore-style': labels.btn_restore_defaults,
            'header-api-settings': labels.header_api_settings,
            'header-palette': labels.header_palette,
            'header-dev-mode': labels.header_dev_mode,
            'header-dict-mgr': labels.header_dict_mgr,
            'btn-dict-clear': labels.btn_clear_all,
            'btn-dict-import': labels.btn_import,
            'btn-dict-export': labels.btn_export,
            'btn-dict-replace': labels.btn_replace,
            'btn-dict-add': labels.btn_add,
            'tab-user': labels.glossary_tab_user || '使用者詞庫',
            'tab-official': labels.glossary_tab_official || '官方推論',
            'page-prev': labels.btn_page_prev || '上一頁',
            'page-next': labels.btn_page_next || '下一頁',
            'label-items': labels.label_items,
            'label-files': labels.label_files,
            'btn-palette-clear-item': labels.btn_restore_defaults || '🗑 清除元件覆寫',
        };

        for (const [id, txt] of Object.entries(mapping)) {
            const el = document.getElementById(id);
            if (el && txt) el.textContent = txt;
        }

        document.querySelectorAll('label[for]').forEach((el) => {
            const forId = el.getAttribute('for');
            const underscored = forId.replace(/-/g, '_');
            const key1 = `label_${underscored}`;
            const key2 = underscored;

            if (forId === 'api-provider' && labels.label_provider) el.textContent = labels.label_provider;
            else if (forId === 'selected-model' && labels.label_model) el.textContent = labels.label_model;
            else if (forId === 'batch-max-chars' && labels.label_max_chars) el.textContent = labels.label_max_chars;
            else if (forId === 'timeout-sec' && labels.label_timeout) el.textContent = labels.label_timeout;
            else if (forId === 'glossary-priority' && labels.label_glossary_priority)
                el.textContent = labels.label_glossary_priority;
            else if (forId === 'palette-target-type' && labels.label_palette_target_type)
                el.textContent = labels.label_palette_target_type;
            else if (forId === 'palette-target-item' && labels.label_palette_target_item)
                el.textContent = labels.label_palette_target_item;
            else if (forId === 'palette-property' && labels.label_palette_property)
                el.textContent = labels.label_palette_property;
            else if (forId === 'palette-color' && labels.label_palette_color)
                el.textContent = labels.label_palette_color;
            else if (forId === 'palette-rounding' && labels.label_palette_rounding)
                el.textContent = labels.label_palette_rounding;
            else if (forId === 'user-prompt' && labels.label_user_prompt) el.textContent = labels.label_user_prompt;
            else if (forId === 'system-prompt' && labels.label_system_prompt)
                el.textContent = labels.label_system_prompt;
            else if (forId === 'input-path' && labels.label_input_path) el.textContent = labels.label_input_path;
            else if (forId === 'btn-rounding-value' && labels.label_global_rounding)
                el.textContent = labels.label_global_rounding;
            else if (forId === 'chk-llm-log' && (labels.label_enable_log || labels.label_llm_log))
                el.textContent = labels.label_enable_log || labels.label_llm_log;
            else if (labels[key1]) el.textContent = labels[key1];
            else if (labels[key2]) el.textContent = labels[key2];
        });

        document.querySelectorAll('span[id]').forEach((el) => {
            const id = el.id;
            const underscored = id.replace(/-/g, '_');
            const key1 = underscored.startsWith('label_') ? underscored : `label_${underscored}`;
            const key2 = underscored;
            if (labels[key1]) el.textContent = labels[key1];
            else if (labels[key2]) el.textContent = labels[key2];
        });

        const optionMapping = {
            'glossary-priority': { official: labels.glossary_priority_official, user: labels.glossary_priority_user },
            'palette-target-type': { global: labels.group_batch, specific: labels.group_specific },
            'api-provider': { Ollama: 'Ollama', 無: labels.label_provider_none || '無' },
            'palette-target-item': {
                dark_bg: labels.cat_all_bg,
                dark_label: labels.cat_all_labels,
                dark_text: labels.cat_all_text || labels.label_text_color,
                dark_btn_bg: labels.cat_all_buttons,
                dark_btn_text: labels.cat_all_btn_text,
                dark_input_bg: labels.cat_all_inputs,
                dark_list_bg: labels.cat_all_logs,
                dark_tab_active: labels.cat_all_tab_active,
                dark_tab_inactive: labels.cat_all_tab_inactive,
                'btn-translate': labels.spec_btn_run_trans,
                'btn-pause': labels.spec_btn_pause,
                'btn-stop': labels.spec_btn_stop,
                'btn-browse-file': labels.spec_btn_select_file,
                'btn-browse-dir': labels.spec_btn_select_folder,
                'btn-browse-output': labels.spec_btn_output_dir,
                'btn-browse-output-open': labels.spec_btn_open_output,
                'user-prompt': labels.label_user_prompt,
                'system-prompt': labels.label_system_prompt,
                'input-path': labels.label_input_path,
                'output-dir': labels.spec_label_output,
                'dict-dialog': labels.spec_area_dict,
                'log-output': labels.label_log_area,
                'progress-bar': labels.spec_progress_current,
                'batch-progress-bar': labels.spec_progress_total,
            },
            'palette-property': {
                bg: labels.label_bg_color,
                text: labels.label_text_color,
                rounding: labels.label_custom_rounding,
            },
        };

        for (const [selectId, optionsDict] of Object.entries(optionMapping)) {
            const selectEl = document.getElementById(selectId);
            if (!selectEl) continue;
            for (const [val, txt] of Object.entries(optionsDict)) {
                if (!txt) continue;
                const opt = selectEl.querySelector(`option[value="${val}"]`);
                if (opt) opt.textContent = txt;
            }
        }

        if (labels.spec_btn_nav_settings && btnNavApi) btnNavApi.title = labels.spec_btn_nav_settings;
        if (labels.spec_btn_nav_dict && btnNavDict) btnNavDict.title = labels.spec_btn_nav_dict;
        if (labels.spec_btn_nav_palette && btnNavPalette) btnNavPalette.title = labels.spec_btn_nav_palette;
        if (labels.spec_btn_nav_theme && btnNavTheme) btnNavTheme.title = labels.spec_btn_nav_theme;
        if (labels.spec_btn_nav_dev && btnNavDev) btnNavDev.title = labels.spec_btn_nav_dev;

        document.querySelectorAll('input[placeholder], textarea[placeholder]').forEach((el) => {
            const id = el.id;
            if (!id) return;
            const underscored = id.replace(/-/g, '_');
            const key = `placeholder_${underscored}`;
            if (id === 'dict-search' && labels.placeholder_search_terms)
                el.placeholder = labels.placeholder_search_terms;
            else if (id === 'dict-input-key' && labels.placeholder_dict_key)
                el.placeholder = labels.placeholder_dict_key;
            else if (id === 'dict-input-value' && labels.placeholder_dict_value)
                el.placeholder = labels.placeholder_dict_value;
            else if (id === 'input-path' && labels.placeholder_input_path)
                el.placeholder = labels.placeholder_input_path;
            else if (labels[key]) el.placeholder = labels[key];
        });

        const normValue = (v) => (v ? v.replace(/\r\n/g, '\n').trim() : '');

        const knownUserDefaults = [
            '你是一位專業的 Minecraft 模組翻譯員。現在請將以下模組字串翻譯為「繁體中文 (zh_tw)」。\n保持專業的遊戲術語風格（如方塊、實體、附魔）。',
            "You are a professional Minecraft mod translator. Please translate the following strings into 'English (en_us)'.\nMaintain a gaming terminology style (e.g., Block, Entity, Enchantment).",
            '你是一位专业的 Minecraft 模组翻译员。现在请将以下模组字串翻译为「简体中文 (zh_cn)」。\n保持专业的游戏术语风格（如方块、实体、附魔）。',
            'あなたはプロのMinecraft Mod翻訳者です。以下のMod文字列を「日本語 (ja_jp)」に翻訳してください。\nゲームの用語スタイル（例：ブロック、エンティティ、エンチャント）を維持してください。',
        ];
        const knownSysDefaults = [
            '\n\n[內部技術指令 - 請務必遵守]\n1. 僅針對 %%VAR_n%%, %%MC_n%%, %%HEX_n%% 等技術佔位符執行「保持原樣」操作（不可修改、翻譯或增刪標籤）。\n2. 除上述佔位符外的其餘文本內容均「必須」按要求翻譯，絕對不可將全文原樣輸出。',
            '\n\n[Internal Technical Instruction - Must Comply]\n1. Keep technical placeholders like %%VAR_n%%, %%MC_n%%, %%HEX_n%% exactly as they are (do not modify, translate, or add/remove tags).\n2. All other text contents MUST be translated and not output as is.',
            '\n\n[内部技术指令 - 请务必遵守]\n1. 仅针对 %%VAR_n%%, %%MC_n%%, %%HEX_n%% 等技术占位符执行「保持原样」操作（不可修改、翻译或增删标签）。\n2. 除上述占位符外的其余文本内容均「必须」按要求翻译，绝对不可将全文原样输出。',
            '\n\n[内部技術指令 - 必ず遵守してください]\n1. %%VAR_n%%、%%MC_n%%、%%HEX_n%% などの技術プレースホルダーは、そのまま維持してください（変更、翻訳、タグの追加/削除は不可）。\n2. それ以外のすべてのテキストコンテンツは必ず翻訳し、そのまま出力してはいけません。',
        ];

        if (userPrompt && labels.default_user_prompt) {
            const currentVal = normValue(userPrompt.value);
            if (knownUserDefaults.some((v) => normValue(v) === currentVal)) {
                userPrompt.value = labels.default_user_prompt;
            }
        }
        if (systemPrompt && labels.default_system_prompt) {
            const currentVal = normValue(systemPrompt.value);
            if (knownSysDefaults.some((v) => normValue(v) === currentVal)) {
                systemPrompt.value = labels.default_system_prompt;
            }
        }

        const selectedModel = document.getElementById('selected-model');
        if (selectedModel && selectedModel.options.length > 0) {
            const firstOpt = selectedModel.options[0];
            if (firstOpt.value === "") {
                const oldSelect = oldLabels.prompt_select_model || '請選取模型';
                const oldLoading = oldLabels.label_loading_models || '載入中...';
                const oldNoModels = oldLabels.label_no_models || '(無可用模型)';
                if (firstOpt.textContent === oldSelect || firstOpt.textContent === oldLoading || firstOpt.textContent === oldNoModels) {
                    firstOpt.textContent = labels.prompt_select_model || '請選取模型';
                }
            }
        }
    } catch (err) {
        console.error('更新介面語言失敗', err);
    }
}
