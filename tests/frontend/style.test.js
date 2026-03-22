import { describe, it, expect, beforeEach, beforeAll, vi } from 'vitest';

describe('style.js 樣式與主題管理模組', () => {
    let mockInvoke;
    let styleModule;
    let stateModule;

    beforeAll(async () => {
        // 1. Mock Tauri API
        mockInvoke = vi.fn();
        globalThis.window = {
            __TAURI__: {
                core: { invoke: mockInvoke }
            }
        };

        // 2. 動態載入
        styleModule = await import('../../frontend/modules/style.js');
        stateModule = await import('../../frontend/modules/state.js');
    });

    beforeEach(() => {
        // 模擬 DOM 結構與 CSS 變數容器
        document.body.innerHTML = `
            <input id="color-bg" />
            <input id="color-text" />
            <input id="color-btn-bg" />
            <input id="color-btn-text" />
            <input id="font-size" />
            <input id="chk-btn-rounding" type="checkbox" />
            <input id="btn-rounding-value" />
            <input id="chk-pulse" type="checkbox" />
            <input id="pulse-speed" />

            <!-- Palette 控制組 -->
            <select id="palette-target-type">
                <option value="global">全域</option>
                <option value="specific">指定元件</option>
            </select>
            <select id="palette-target-item">
                <option value="dark_bg">背景顏色</option>
                <option value="btn-translate">翻譯按鈕</option>
            </select>
            <select id="palette-property">
                <option value="bg">背景</option>
                <option value="text">文字</option>
                <option value="rounding">圓角</option>
            </select>
            <div id="palette-clear-group"></div>
            <div id="palette-property-group"></div>
            <div id="palette-color-group"></div>
            <div id="palette-rounding-group"></div>
            <input id="palette-color" />
            <input id="palette-rounding" />

            <button id="btn-translate"></button>
            <div id="progress-bar"></div>
        `;

        // 重設 CSS 變數
        document.documentElement.style.removeProperty('--bg-color');
        document.documentElement.style.removeProperty('--text-color');

        mockInvoke.mockReset();
        stateModule.state.currentStyle = {};
        stateModule.state.currentLabels = { label_bg_color: '背景色', label_text_color: '文字色' };
    });

    describe('applyColors', () => {
        it('應該正確將樣式設定注入到 :root 的 CSS 變數中', () => {
            const fakeStyle = {
                theme: 'dark',
                dark_bg: [30, 30, 35],
                dark_text: [255, 255, 255]
            };

            styleModule.applyColors(fakeStyle);

            expect(document.documentElement.style.getPropertyValue('--bg-color')).toBe('rgb(30,30,35)');
            expect(document.documentElement.style.getPropertyValue('--text-color')).toBe('rgb(255,255,255)');
        });

        it('應該正確處理 instance_overrides 並套用行內樣式', () => {
            const fakeStyle = {
                theme: 'dark',
                instance_overrides: {
                    'btn-translate': { bg: [255, 0, 0], text: [0, 0, 0], rounding: 8 }
                }
            };

            styleModule.applyColors(fakeStyle);

            const btn = document.getElementById('btn-translate');
            expect(btn.style.backgroundColor).toBe('rgb(255, 0, 0)');
            expect(btn.style.color).toBe('rgb(0, 0, 0)');
            expect(btn.style.borderRadius).toBe('8px');
        });
    });

    describe('loadStyle', () => {
        it('應該從後端下載樣式配置並更新 input 常數', async () => {
            const fakeStyle = {
                theme: 'dark',
                dark_bg: [30, 30, 35],
                font_size: 16,
                btn_rounding_enabled: true,
                btn_rounding_value: 5.0
            };
            mockInvoke.mockResolvedValue(fakeStyle);

            await styleModule.loadStyle();

            expect(document.getElementById('color-bg').value).toBe('#1e1e23'); // rgbToHex(30,30,35)
            expect(document.getElementById('font-size').value).toBe('16');
            expect(document.getElementById('chk-btn-rounding').checked).toBe(true);
            expect(document.getElementById('btn-rounding-value').value).toBe('5');
        });
    });

    describe('updatePaletteValue', () => {
        it('非指定元件（全域）時，應隱含屬性選單並正確帶入顏色', () => {
            stateModule.state.currentStyle = {
                dark_bg: [15, 15, 20]
            };

            const targetType = document.getElementById('palette-target-type');
            const targetItem = document.getElementById('palette-target-item');
            targetType.value = 'global';
            targetItem.value = 'dark_bg';

            styleModule.updatePaletteValue();

            // 驗證區塊開關
            expect(document.getElementById('palette-property-group').style.display).toBe('none');
            expect(document.getElementById('palette-color-group').style.display).toBe('block');
            
            // 驗證色值帶入
            expect(document.getElementById('palette-color').value).toBe('#0f0f14');
        });
    });
    describe('saveStyle', () => {
        it('應該讀取 DOM 數值並調用 save_style_config', async () => {
            stateModule.state.currentStyle = { theme: 'dark' };

            document.getElementById('font-size').value = '20';
            document.getElementById('chk-btn-rounding').checked = true;
            document.getElementById('btn-rounding-value').value = '10';

            await styleModule.saveStyle();

            expect(stateModule.state.currentStyle.font_size).toBe(20);
            expect(stateModule.state.currentStyle.btn_rounding_enabled).toBe(true);
            expect(stateModule.state.currentStyle.btn_rounding_value).toBe(10);
            expect(mockInvoke).toHaveBeenCalledWith('save_style_config', { config: expect.any(Object) });
        });
    });

    describe('updatePaletteValue - 進階', () => {
        it('指定元件且屬性為圓角時，應顯示圓角控制群組', () => {
            stateModule.state.currentStyle = { instance_overrides: { 'btn-translate': { rounding: 12 } } };

            const targetType = document.getElementById('palette-target-type');
            const targetItem = document.getElementById('palette-target-item');
            const property = document.getElementById('palette-property');
            targetType.value = 'specific';
            targetItem.value = 'btn-translate';
            property.value = 'rounding';

            styleModule.updatePaletteValue();

            expect(document.getElementById('palette-rounding-group').style.display).toBe('block');
            expect(document.getElementById('palette-color-group').style.display).toBe('none');
            expect(document.getElementById('palette-rounding').value).toBe('12');
        });

        it('進度條元件應該隱藏文字選項', () => {
            const targetItem = document.getElementById('palette-target-item');
            const opt = document.createElement('option');
            opt.value = 'progress-bar';
            targetItem.appendChild(opt);

            const targetType = document.getElementById('palette-target-type');
            const property = document.getElementById('palette-property');
            
            targetType.value = 'specific';
            targetItem.value = 'progress-bar';


            styleModule.updatePaletteValue();

            const textOpt = Array.from(property.options).find(opt => opt.value === 'text');
            expect(textOpt.style.display).toBe('none');
        });
    });
});

