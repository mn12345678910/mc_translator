import { describe, it, expect, beforeEach, beforeAll, vi } from 'vitest';

describe('utils.js 工具模組', () => {
    let utilsModule;

    beforeAll(async () => {
        // 動態載入
        utilsModule = await import('../../frontend/modules/utils.js');

        // 模擬視窗定時器，用於測試 debounce
        vi.useFakeTimers();
    });

    beforeEach(() => {
        document.body.innerHTML = `
            <div id="log-output" style="height: 100px; overflow-y: auto;"></div>
        `;
    });

    describe('rgbToHex', () => {
        it('應該將 [255, 0, 0] 轉換為 #ff0000', () => {
            expect(utilsModule.rgbToHex([255, 0, 0])).toBe('#ff0000');
        });

        it('應該將 [0, 255, 0] 轉換為 #00ff00', () => {
            expect(utilsModule.rgbToHex([0, 255, 0])).toBe('#00ff00');
        });

        it('應該對短陣列回傳預設色', () => {
            expect(utilsModule.rgbToHex([255])).toBe('#333333');
            expect(utilsModule.rgbToHex(null)).toBe('#333333');
        });
    });

    describe('hexToRgb', () => {
        it('應該將 #ff0000 轉換為 [255, 0, 0]', () => {
            expect(utilsModule.hexToRgb('#ff0000')).toEqual([255, 0, 0]);
        });

        it('應該將 #0000ff 轉換為 [0, 0, 255]', () => {
            expect(utilsModule.hexToRgb('#0000ff')).toEqual([0, 0, 255]);
        });
    });

    describe('debounce 防抖動', () => {
        it('在指定延遲內多次觸發，應該只執行一次', () => {
            const callback = vi.fn();
            const debounced = utilsModule.debounce(callback, 200);

            // 連續呼叫 3 次
            debounced('a');
            debounced('b');
            debounced('c');

            // 尚未到 200ms -> 應該沒執行
            expect(callback).not.toHaveBeenCalled();

            // 時間快進 200ms
            vi.advanceTimersByTime(200);

            // 應該已被呼叫 1 次，且帶入最後一次參數 'c'
            expect(callback).toHaveBeenCalledTimes(1);
            expect(callback).toHaveBeenCalledWith('c');
        });
    });

    describe('appendLog 日誌紀錄', () => {
        beforeEach(() => {
            // 模擬全域 __logViewer
            window.__logViewer = {
                appendLog: vi.fn(),
            };
        });

        it('應將訊息轉送給全域的 __logViewer', () => {
            utilsModule.appendLog('測試一則訊息');
            expect(window.__logViewer.appendLog).toHaveBeenCalledWith(
                '測試一則訊息',
                'info',
                expect.any(String),
                []
            );
        });

        it('當訊息包含 ❌ 或 Error 時，應自動識別為 error 等級', () => {
            utilsModule.appendLog('❌ 發生錯誤');
            expect(window.__logViewer.appendLog).toHaveBeenCalledWith('❌ 發生錯誤', 'error', expect.any(String), []);
        });

        it('應正確處理對象形式的 entry', () => {
            utilsModule.appendLog({
                level: 'Success',
                message: '完成',
                timestamp: Date.now(),
                segments: [{ kind: 'text', text: '完成' }],
            });
            expect(window.__logViewer.appendLog).toHaveBeenCalledWith('完成', 'success', expect.any(String), [
                { kind: 'text', text: '完成' },
            ]);
        });

        it('當 __logViewer 未初始化時應輸出警告並回傳', () => {
            const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
            const orig = window.__logViewer;
            delete window.__logViewer;
            utilsModule.appendLog('test');
            expect(warnSpy).toHaveBeenCalledWith('Log viewer not yet initialized');
            window.__logViewer = orig;
            warnSpy.mockRestore();
        });
    });

    describe('escapeHtml HTML 跳脫', () => {
        it('應該正確跳脫 HTML 字元', () => {
            expect(utilsModule.escapeHtml('<script>')).toBe('&lt;script&gt;');
        });

        it('當傳入空值、null 或 undefined 應回傳空字串', () => {
            expect(utilsModule.escapeHtml(null)).toBe('');
            expect(utilsModule.escapeHtml(undefined)).toBe('');
            expect(utilsModule.escapeHtml('')).toBe('');
        });
    });
});
