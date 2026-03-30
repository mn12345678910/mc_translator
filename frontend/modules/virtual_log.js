// frontend/modules/virtual_log.js
import { prepare, layout } from '@chenglou/pretext';

/**
 * 基於 Pretext 的虛擬化日誌檢視器
 * 核心原理：預先測量所有文字行高度，僅渲染可視區域內的 DOM 節點。
 */
export class VirtualLogViewer {
    constructor(containerId, options = {}) {
        this.container = document.getElementById(containerId);
        if (!this.container) throw new Error(`Container #${containerId} not found`);

        this.options = {
            lineHeight: options.lineHeight || 18,
            fontSize: options.fontSize || '13px',
            fontFamily: options.fontFamily || "'Courier New', Courier, monospace",
            buffer: options.buffer || 5, // 上下快取渲染行數
            ...options,
        };

        this.logs = []; // 儲存格式: { message, level, timestamp, height }
        this.isLockedToBottom = true;
        this.itemHeights = [];
        this.totalHeight = 0;

        this.init();
    }

    init() {
        // 設定容器樣式
        this.container.style.position = 'relative';
        this.container.style.overflowY = 'auto';

        // 建立捲動佔位符 (Scroller)
        this.scroller = document.createElement('div');
        this.scroller.className = 'log-scroller';
        this.scroller.style.width = '100%';
        this.scroller.style.height = '0px';
        this.container.appendChild(this.scroller);

        // 建立可視內容容器 (Viewport Container)
        this.viewport = document.createElement('div');
        this.viewport.className = 'log-viewport';
        this.viewport.style.position = 'absolute';
        this.viewport.style.top = '0';
        this.viewport.style.left = '0';
        this.viewport.style.width = '100%';
        this.viewport.style.padding = '10px';
        this.viewport.style.boxSizing = 'border-box';
        this.container.appendChild(this.viewport);

        // 監聽捲動與尺寸變化
        this.container.addEventListener('scroll', () => this.handleScroll());
        this.resizeObserver = new ResizeObserver(() => {
            this.recalculateHeights();
            this.render();
        });
        this.resizeObserver.observe(this.container);
    }

    /**
     * 新增一條日誌
     */
    appendLog(message, level = 'info', timeStr = '') {
        const width = this.container.clientWidth - 20; // 扣除 padding
        const height = this.measureHeight(message, width);

        const logEntry = {
            message,
            level,
            timeStr,
            height,
        };

        this.logs.push(logEntry);
        this.itemHeights.push(height);
        this.totalHeight += height;

        this.scroller.style.height = `${this.totalHeight}px`;

        if (this.isLockedToBottom) {
            this.container.scrollTop = this.totalHeight;
        }

        this.render();
    }

    measureHeight(text, width) {
        // 使用 pretext 進行文字折行測量
        const font = `${this.options.fontSize} ${this.options.fontFamily}`;
        const prepared = prepare(text, font);
        const result = layout(prepared, width, this.options.lineHeight);
        return Math.max(this.options.lineHeight, result.height);
    }

    recalculateHeights() {
        const width = this.container.clientWidth - 20;
        this.totalHeight = 0;
        this.itemHeights = this.logs.map((log) => {
            const h = this.measureHeight(log.message, width);
            log.height = h;
            this.totalHeight += h;
            return h;
        });
        this.scroller.style.height = `${this.totalHeight}px`;
    }

    handleScroll() {
        const { scrollTop, scrollHeight, clientHeight } = this.container;
        // 判斷是否鎖定在底部 (保留 30px 的誤觸空間)
        this.isLockedToBottom = scrollHeight - scrollTop - clientHeight < 30;
        this.render();
    }

    render() {
        const scrollTop = this.container.scrollTop;
        const viewHeight = this.container.clientHeight;

        // 計算哪些行在可視區域內
        let currentY = 0;
        let startIndex = -1;
        let endIndex = -1;

        for (let i = 0; i < this.itemHeights.length; i++) {
            const h = this.itemHeights[i];
            if (startIndex === -1 && currentY + h > scrollTop) {
                startIndex = Math.max(0, i - this.options.buffer);
            }
            if (currentY > scrollTop + viewHeight) {
                endIndex = Math.min(this.logs.length - 1, i + this.options.buffer);
                break;
            }
            currentY += h;
        }

        if (startIndex === -1) startIndex = 0;
        if (endIndex === -1) endIndex = this.logs.length - 1;

        // 重新計算區塊精確 Top 位移
        let exactTop = 0;
        for (let i = 0; i < startIndex; i++) {
            exactTop += this.itemHeights[i];
        }

        this.viewport.style.transform = `translateY(${exactTop}px)`;
        this.renderSlice(startIndex, endIndex);
    }

    renderSlice(start, end) {
        const fragment = document.createDocumentFragment();
        for (let i = start; i <= end; i++) {
            const log = this.logs[i];
            const div = document.createElement('div');
            div.className = `log-line log-${log.level}`;
            div.style.height = `${log.height}px`;

            const timeSpan = document.createElement('span');
            timeSpan.className = 'log-time';
            timeSpan.textContent = log.timeStr ? `[${log.timeStr}] ` : '';

            const msgSpan = document.createElement('span');
            msgSpan.className = 'log-msg';
            msgSpan.textContent = log.message;

            div.appendChild(timeSpan);
            div.appendChild(msgSpan);
            fragment.appendChild(div);
        }
        this.viewport.innerHTML = '';
        this.viewport.appendChild(fragment);
    }
}
