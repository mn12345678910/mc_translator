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
        it('應在 #log-output 新增 <p> 元素', () => {
            utilsModule.appendLog('測試一則訊息');

            const logOutput = document.getElementById('log-output');
            expect(logOutput.childNodes.length).toBe(1);

            const logLine = logOutput.querySelector('p');
            expect(logLine.textContent).toContain('測試一則訊息');
        });

        it('當訊息包含 ❌ 或 Error 時，顏色應為紅色', () => {
            utilsModule.appendLog('❌ 發生錯誤');

            const logOutput = document.getElementById('log-output');
            const logLine = logOutput.querySelector('p');
            // 相容 Happy DOM 回傳 #ff6b6b 或 瀏覽器回傳 rgb
            expect(['#ff6b6b', 'rgb(255, 107, 107)']).toContain(logLine.style.color);
        });

        it('當日誌數量超過 500，應刪除最舊的項目', () => {
            const logOutput = document.getElementById('log-output');

            // 灌入 505 筆
            for(let i=1; i<=505; i++) {
                utilsModule.appendLog(`訊息 ${i}`);
            }

            expect(logOutput.childNodes.length).toBe(500);

            // 最舊的應該已經被刪除，第一個節點應該不會是 "訊息 1"
            expect(logOutput.firstChild.textContent).not.toContain('訊息 1');
            expect(logOutput.lastChild.textContent).toContain('訊息 505');
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
