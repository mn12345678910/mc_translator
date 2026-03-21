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

    it('updateUiLanguage 應該在 User Prompt 符合預設簡體提示字串時，更新它', async () => {
        const mockLabels = {
            default_user_prompt: 'English prompt defaults...',
        };
        mockInvoke.mockResolvedValue(mockLabels);

        const promptArea = document.getElementById('user-prompt');
        // 加入含有「风格」的簡體中文提示字串（已被我們修好的狀態）
        promptArea.value = '你是一位专业的 Minecraft 模组翻译员。现在请将以下模组字串翻译为「简体中文 (zh_cn)」。\n保持专业的游戏术语风格（如方块、实体、附魔）。';

        await i18nModule.updateUiLanguage();

        // 畫面應該更新為 English 預設
        expect(promptArea.value.trim()).toBe('English prompt defaults...');
    });
});
