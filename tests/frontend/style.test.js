import { describe, it, expect, beforeEach, beforeAll, vi } from 'vitest';

describe('style.js 樣式與主題管理模組', () => {
    let mockInvoke;
    let styleModule;
    let stateModule;

    beforeAll(async () => {
        vi.resetModules();
        mockInvoke = vi.fn();
        globalThis.mockInvoke = mockInvoke; // 對接 tauri_mock.js
        globalThis.window = {
            __TAURI__: {
                core: { invoke: mockInvoke },
                event: { listen: vi.fn(() => Promise.resolve(() => {})) },
            },
        };
        styleModule = await import('../../frontend/modules/style.js');
        stateModule = await import('../../frontend/modules/state.js');
    });

    beforeEach(() => {
        // 模擬 DOM 結構與 CSS 變數容器
        document.body.innerHTML = `
            <input id="color-bg" />
            <input id="color-text" />
            <input id="color-accent" />
            <input id="color-danger" />
            <input id="font-size" />
            <input id="chk-btn-rounding" type="checkbox" />
            <input id="btn-rounding-value" />
            <input id="chk-pulse" type="checkbox" />
            <input id="pulse-speed" />
            <select id="progress-style"></select>

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
            <div id="palette-number-group"></div>
            <input id="palette-color" />
            <input id="palette-number" />

            <button id="btn-translate"></button>
            <div id="progress-bar"></div>
        `;

        // 重設 CSS 變數
        document.documentElement.style.removeProperty('--bg-color');
        document.documentElement.style.removeProperty('--text-color');

        // 手動加載 label-palette-color 避免 uncovered 210
        document.body.innerHTML += `<div id="label-palette-color"></div>`;

        mockInvoke.mockReset();
        mockInvoke.mockImplementation(async (cmd) => {
            if (cmd === 'derive_palette_state_cmd') {
                return {
                    show_clear_group: false,
                    show_property_group: false,
                    show_color_group: true,
                    show_number_group: false,
                    label_palette_number: 'Number',
                    label_palette_color: 'Color',
                    number_value: 0,
                    number_step: 1,
                    color_value: '#ffffff',
                };
            }
            if (cmd === 'build_style_from_form_cmd') {
                return {
                    ...(stateModule.state.currentStyle || {}),
                    font_size: parseFloat(document.getElementById('font-size')?.value || '16'),
                    btn_rounding_enabled: !!document.getElementById('chk-btn-rounding')?.checked,
                    btn_rounding_value: parseFloat(document.getElementById('btn-rounding-value')?.value || '4'),
                    progress_pulse_enabled: !!document.getElementById('chk-pulse')?.checked,
                    progress_pulse_speed: parseFloat(document.getElementById('pulse-speed')?.value || '1'),
                    progress_style: document.getElementById('progress-style')?.value || 'default',
                    dark_bg: document.getElementById('color-bg')?.value === '#ff0000' ? [255, 0, 0] : [30, 30, 35],
                    dark_text: document.getElementById('color-text')?.value === '#00ff00' ? [0, 255, 0] : [255, 255, 255],
                };
            }
            if (cmd === 'get_gui_css_vars') return {};
            return null;
        });
        stateModule.state.currentStyle = {};
        stateModule.state.currentLabels = { label_bg_color: '背景色', label_text_color: '文字色' };
    });

    describe('applyColors', () => {
        it('應該正確將樣式設定注入到 :root 的 CSS 變數中', () => {
            const fakeStyle = {
                theme: 'dark',
                dark_bg: [30, 30, 35],
                dark_text: [255, 255, 255],
                dark_btn_bg: [0, 100, 200],
                dark_btn_text: [255, 255, 255],
                dark_input_bg: [10, 20, 30],
                dark_list_bg: [40, 50, 60],
                dark_tab_active: [70, 80, 90],
                dark_tab_inactive: [100, 110, 120],
                dark_label: [130, 140, 150],
            };

            styleModule.applyColors(fakeStyle);

            expect(document.documentElement.style.getPropertyValue('--bg-color')).toBe('rgb(30,30,35)');
            expect(document.documentElement.style.getPropertyValue('--text-color')).toBe('rgb(255,255,255)');
        });

        it('應該正確處理 instance_overrides 並套用行內樣式', () => {
            const fakeStyle = {
                theme: 'dark',
                instance_overrides: {
                    'btn-translate': { dark_bg: [255, 0, 0], dark_text: [0, 0, 0], rounding: 8 },
                },
            };

            styleModule.applyColors(fakeStyle);

            const btn = document.getElementById('btn-translate');
            expect(btn.style.backgroundColor).toBe('rgb(255, 0, 0)');
            expect(btn.style.color).toBe('rgb(0, 0, 0)');
            expect(btn.style.borderRadius).toBe('8px');
        });

        it('切換主題時應該正確選擇對應的主題覆寫色', () => {
            const fakeStyle = {
                theme: 'light',
                instance_overrides: {
                    'btn-translate': {
                        dark_bg: [0, 0, 0],
                        light_bg: [255, 255, 255],
                    },
                },
            };

            styleModule.applyColors(fakeStyle);
            const btn = document.getElementById('btn-translate');
            expect(btn.style.backgroundColor).toBe('rgb(255, 255, 255)');
        });
    });

    describe('loadStyle', () => {
        it('應該從後端下載樣式配置並更新 input 常數', async () => {
            const fakeStyle = {
                theme: 'dark',
                dark_bg: [30, 30, 35],
                font_size: 16,
                btn_rounding_enabled: true,
                btn_rounding_value: 5.0,
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
        it('非指定元件（全域）時，應隱含屬性選單並正確帶入顏色', async () => {
            stateModule.state.currentStyle = {
                dark_bg: [15, 15, 20],
            };
            mockInvoke.mockImplementation(async (cmd) => {
                if (cmd === 'derive_palette_state_cmd') {
                    return {
                        show_clear_group: false,
                        show_property_group: false,
                        show_color_group: true,
                        show_number_group: false,
                        label_palette_number: 'Number',
                        label_palette_color: '背景色',
                        number_value: 0,
                        number_step: 1,
                        color_value: '#0f0f14',
                    };
                }
                if (cmd === 'get_gui_css_vars') return {};
                return null;
            });

            const targetType = document.getElementById('palette-target-type');
            const targetItem = document.getElementById('palette-target-item');
            targetType.value = 'global';
            targetItem.value = 'dark_bg';

            await styleModule.updatePaletteValue();

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
        it('指定元件且屬性為圓角時，應顯示圓角控制群組', async () => {
            stateModule.state.currentStyle = { instance_overrides: { 'btn-translate': { rounding: 12 } } };
            mockInvoke.mockImplementation(async (cmd) => {
                if (cmd === 'derive_palette_state_cmd') {
                    return {
                        show_clear_group: true,
                        show_property_group: true,
                        show_color_group: false,
                        show_number_group: true,
                        label_palette_number: 'Rounding (px)',
                        label_palette_color: '背景色',
                        number_value: 12,
                        number_step: 1,
                        color_value: '#ffffff',
                    };
                }
                if (cmd === 'get_gui_css_vars') return {};
                return null;
            });

            const targetType = document.getElementById('palette-target-type');
            const targetItem = document.getElementById('palette-target-item');
            const property = document.getElementById('palette-property');
            targetType.value = 'specific';
            targetItem.value = 'btn-translate';
            property.value = 'rounding';

            await styleModule.updatePaletteValue();

            expect(document.getElementById('palette-number-group').style.display).toBe('block');
            expect(document.getElementById('palette-color-group').style.display).toBe('none');
            expect(document.getElementById('palette-number').value).toBe('12');
        });

        it('進度條元件應該隱藏文字選項', async () => {
            const targetItem = document.getElementById('palette-target-item');
            const opt = document.createElement('option');
            opt.value = 'progress-bar';
            targetItem.appendChild(opt);

            const targetType = document.getElementById('palette-target-type');
            const property = document.getElementById('palette-property');

            targetType.value = 'specific';
            targetItem.value = 'progress-bar';

            await styleModule.updatePaletteValue();

            const textOpt = Array.from(property.options).find((opt) => opt.value === 'text');
            expect(textOpt.style.display).toBe('none');
        });

        it('當選擇 progress-bar 且屬性為 text 時應該 fallback 為 bg', async () => {
            const targetType = document.getElementById('palette-target-type');
            const targetItem = document.getElementById('palette-target-item');
            const property = document.getElementById('palette-property');

            targetType.value = 'specific';
            // 手動加入進度條選項
            const opt = document.createElement('option');
            opt.value = 'progress-bar';
            targetItem.appendChild(opt);
            targetItem.value = 'progress-bar';

            property.value = 'text';

            await styleModule.updatePaletteValue();

            expect(property.value).toBe('bg');
        });
    });

    describe('applyColors - 進階', () => {
        it('應該在 rounding_enabled 為 false 時設定 border-radius 為 0px', () => {
            const fakeStyle = { theme: 'dark', btn_rounding_enabled: false };
            styleModule.applyColors(fakeStyle);
            expect(document.documentElement.style.getPropertyValue('--border-radius')).toBe('0px');
        });

        it('應該在啟用脈動且進度條存在時設定 animation', () => {
            const fakeStyle = { theme: 'dark', progress_pulse_enabled: true, progress_pulse_speed: 2.0 };
            styleModule.applyColors(fakeStyle);
            const progressBar = document.getElementById('progress-bar');
            expect(progressBar.style.animation).toContain('pulse');
        });

        it('應該在啟用脈動但進度條不存在時不拋出異常', () => {
            const fakeStyle = { theme: 'dark', progress_pulse_enabled: true, progress_pulse_speed: 2.0 };
            document.getElementById('progress-bar').remove();
            expect(() => styleModule.applyColors(fakeStyle)).not.toThrow();
        });
    });

    describe('loadStyle 異常處理', () => {
        it('invoke 失敗時應該捕獲異常並輸出錯誤', async () => {
            mockInvoke.mockRejectedValue(new Error('Style fetch failed'));
            const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

            await styleModule.loadStyle();

            expect(consoleErrorSpy).toHaveBeenCalled();
            consoleErrorSpy.mockRestore();
        });
    });

    describe('saveStyle 顏色讀取', () => {
        it('應該從顏色選擇器讀取 hex 並轉換為 RGB', async () => {
            stateModule.state.currentStyle = { theme: 'dark' };
            document.getElementById('color-bg').value = '#ff0000';
            document.getElementById('color-text').value = '#00ff00';

            await styleModule.saveStyle();

            expect(stateModule.state.currentStyle.dark_bg).toEqual([255, 0, 0]);
            expect(stateModule.state.currentStyle.dark_text).toEqual([0, 255, 0]);
        });
    });

    describe('restoreDefaultStyle', () => {
        it('應該從後端獲取預設樣式並套用', async () => {
            const defaultStyle = {
                theme: 'light',
                font_size: 14,
                btn_rounding_enabled: true,
                btn_rounding_value: 4,
                progress_pulse_enabled: false,
                progress_pulse_speed: 1,
                progress_style: 'default',
                dark_bg: [30, 30, 35],
                dark_text: [255, 255, 255],
                dark_accent: [0, 100, 200],
                dark_danger: [200, 0, 0],
            };
            mockInvoke.mockImplementation(async (cmd) => {
                if (cmd === 'get_default_style_config') return defaultStyle;
                return {};
            });
            stateModule.state.currentStyle = { show_palette_settings: true };

            await styleModule.restoreDefaultStyle();

            expect(stateModule.state.currentStyle.font_size).toBe(14);
            expect(stateModule.state.currentStyle.show_palette_settings).toBe(true);
            expect(mockInvoke).toHaveBeenCalledWith('save_style_config', { config: expect.any(Object) });
        });

        it('invoke 失敗時應該捕獲異常', async () => {
            mockInvoke.mockRejectedValue(new Error('Restore failed'));
            const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

            await styleModule.restoreDefaultStyle();

            expect(consoleErrorSpy).toHaveBeenCalled();
            consoleErrorSpy.mockRestore();
        });
    });
});
