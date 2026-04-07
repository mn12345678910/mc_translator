import { describe, it, expect, beforeEach, vi } from 'vitest';
import { VirtualLogViewer } from '../../frontend/modules/virtual_log.js';

// Mock ResizeObserver
global.ResizeObserver = class {
    constructor(callback) {
        this.callback = callback;
    }
    observe() {}
    unobserve() {}
    disconnect() {}
};

// Mock requestAnimationFrame
global.requestAnimationFrame = (cb) => setTimeout(cb, 0);

describe('VirtualLogViewer 完整覆蓋', () => {
    let container;
    let viewer;

    beforeEach(() => {
        document.body.innerHTML = '<div id="log-container" style="height: 400px; width: 600px; overflow: auto;"></div>';
        container = document.getElementById('log-container');
        Object.defineProperty(container, 'clientHeight', { value: 400, configurable: true, writable: true });
        Object.defineProperty(container, 'clientWidth', { value: 600, configurable: true, writable: true });
        Object.defineProperty(container, 'scrollTop', { value: 0, configurable: true, writable: true });
        Object.defineProperty(container, 'scrollHeight', { value: 400, configurable: true, writable: true });

        vi.spyOn(VirtualLogViewer.prototype, 'measureHeight').mockReturnValue(20);

        viewer = new VirtualLogViewer('log-container');
        // Build cumulativeHeights for findIndexAtOffset tests
        viewer.logs = [];
        viewer.itemHeights = [];
        viewer.cumulativeHeights = [];
        viewer.totalHeight = 0;
        for (let i = 0; i < 10; i++) {
            viewer.logs.push({ message: `Log ${i}`, level: 'info', timeStr: '00:00:00', height: 20 });
            viewer.itemHeights.push(20);
            viewer.totalHeight += 20;
            viewer.cumulativeHeights.push(viewer.totalHeight);
        }
        viewer.scroller.style.height = `${viewer.totalHeight + viewer.paddingY}px`;
    });

    describe('findIndexAtOffset 二分搜尋', () => {
        it('offset 為 0 時應該回傳 0', () => {
            expect(viewer.findIndexAtOffset(0)).toBe(0);
        });

        it('offset 在第一個項目範圍內應該回傳 0', () => {
            expect(viewer.findIndexAtOffset(10)).toBe(0);
        });

        it('offset 在第五個項目應該回傳 4', () => {
            expect(viewer.findIndexAtOffset(90)).toBe(4);
        });

        it('offset 超過所有項目應該回傳最後一個索引 + 1', () => {
            expect(viewer.findIndexAtOffset(9999)).toBe(10);
        });
    });

    describe('triggerUpdate', () => {
        it('當有 onUpdate 回呼時應該觸發更新', () => {
            const callback = vi.fn();
            viewer.onUpdate = callback;
            viewer.lastRenderedCount = 5;
            viewer.triggerUpdate();
            expect(callback).toHaveBeenCalledWith({ total: 10, rendered: 5, isLocked: true });
        });

        it('當沒有 onUpdate 回呼時不應該拋出異常', () => {
            viewer.onUpdate = null;
            expect(() => viewer.triggerUpdate()).not.toThrow();
        });
    });

    describe('handleScroll', () => {
        it('使用者捲動後應該設定 suspendAutoScroll', () => {
            viewer.isLockedToBottom = true;
            Object.defineProperty(container, 'scrollTop', { value: 100, configurable: true, writable: true });
            Object.defineProperty(container, 'scrollHeight', { value: 500, configurable: true, writable: true });

            viewer.handleScroll();

            expect(viewer.suspendAutoScroll).toBe(true);
        });

        it('程式觸發的捲動不應該設定 suspendAutoScroll', () => {
            viewer.isProgrammaticScroll = true;
            viewer.handleScroll();
            expect(viewer.suspendAutoScroll).toBe(false);
        });
    });

    describe('syncBottomIfNeeded', () => {
        it('當未鎖定底部時應該直接回傳', () => {
            viewer.isLockedToBottom = false;
            viewer.syncBottomIfNeeded();
            // Should not throw and not scroll
        });

        it('當已鎖定底部且已在底部時不應該捲動', () => {
            viewer.isLockedToBottom = true;
            Object.defineProperty(container, 'scrollTop', { value: 0, configurable: true, writable: true });
            Object.defineProperty(container, 'scrollHeight', { value: 400, configurable: true, writable: true });
            Object.defineProperty(container, 'clientHeight', { value: 400, configurable: true, writable: true });

            viewer.syncBottomIfNeeded();

            expect(container.scrollTop).toBe(0);
        });
    });

    describe('renderSlice 標籤解析', () => {
        it('應該正確解析 <dir> 標籤並套用 log-dir class', () => {
            viewer.logs = [
                { message: '<dir>path/to/dir</dir> some text', level: 'info', timeStr: '00:00:00', height: 20 },
            ];
            viewer.itemHeights = [20];
            viewer.cumulativeHeights = [20];
            viewer.totalHeight = 20;

            viewer.renderSlice(0, 0);

            const dirSpan = viewer.viewport.querySelector('.log-dir');
            expect(dirSpan).toBeTruthy();
            expect(dirSpan.textContent).toBe('path/to/dir');
        });

        it('應該正確解析 <file> 標籤並套用 log-file class', () => {
            viewer.logs = [
                { message: '<file>config.json</file> loaded', level: 'info', timeStr: '00:00:00', height: 20 },
            ];
            viewer.itemHeights = [20];
            viewer.cumulativeHeights = [20];
            viewer.totalHeight = 20;

            viewer.renderSlice(0, 0);

            const fileSpan = viewer.viewport.querySelector('.log-file');
            expect(fileSpan).toBeTruthy();
            expect(fileSpan.textContent).toBe('config.json');
        });

        it('應該正確處理混合標籤和一般文字', () => {
            viewer.logs = [
                {
                    message: 'Start <dir>src/</dir> and <file>main.js</file> end',
                    level: 'info',
                    timeStr: '00:00:00',
                    height: 20,
                },
            ];
            viewer.itemHeights = [20];
            viewer.cumulativeHeights = [20];
            viewer.totalHeight = 20;

            viewer.renderSlice(0, 0);

            expect(viewer.viewport.querySelector('.log-dir')).toBeTruthy();
            expect(viewer.viewport.querySelector('.log-file')).toBeTruthy();
            expect(viewer.viewport.textContent).toContain('Start');
            expect(viewer.viewport.textContent).toContain('end');
        });
    });

    describe('render 空狀態', () => {
        it('當 logs 為空時 render 應該直接回傳', () => {
            viewer.logs = [];
            expect(() => viewer.render()).not.toThrow();
        });
    });
});
