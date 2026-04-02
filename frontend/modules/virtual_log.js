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
        this.isProgrammaticScroll = false;
        this.suspendAutoScroll = false;
        this.lastUserScrollAt = 0;
        this.itemHeights = [];
        this.cumulativeHeights = []; // [NEW] 儲存累加高度，用於 O(log N) 搜尋
        this.totalHeight = 0;
        this.paddingY = 20; // 視圖容器總邊距 (上下各 10px)
        this.onUpdate = options.onUpdate || null; // [NEW] 狀態更新回調
        this.lockThreshold = options.lockThreshold || 30;
        this.userScrollGraceMs = options.userScrollGraceMs || 200;

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
            if (this.isLockedToBottom) {
                this.scrollToBottom();
            }
        });
        this.resizeObserver.observe(this.container);
    }

    scrollToBottom() {
        if (this.suspendAutoScroll) return;
        this.isProgrammaticScroll = true;
        const raf =
            (typeof globalThis !== 'undefined' && globalThis.requestAnimationFrame) || ((cb) => setTimeout(cb, 0));
        raf(() => {
            this.container.scrollTop = Math.max(0, this.container.scrollHeight - this.container.clientHeight);
            this.isProgrammaticScroll = false;
        });
    }

    syncBottomIfNeeded() {
        if (!this.isLockedToBottom || this.suspendAutoScroll) return;
        const gap = this.container.scrollHeight - (this.container.scrollTop + this.container.clientHeight);
        if (gap > 0 && gap <= this.lockThreshold * 2) {
            this.container.scrollTop += gap;
        }
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
        this.cumulativeHeights.push(this.totalHeight + height); // [NEW] 增量更新累加高度
        this.totalHeight += height;

        this.scroller.style.height = `${this.totalHeight + this.paddingY}px`;

        if (this.isLockedToBottom) {
            this.scrollToBottom();
        }

        this.render();
    }

    measureHeight(text, width) {
        // [NEW] 測量前過濾掉所有標籤，確保 pretext 計算的是純文字寬度
        const cleanMsg = text.replace(/<dir>|<\/dir>|<file>|<\/file>/g, '');
        const font = `${this.options.fontSize} ${this.options.fontFamily}`;
        const safetyWidth = Math.max(10, width - 4); // [FIX] 增加安全寬度緩衝
        const prepared = prepare(cleanMsg, font);
        const result = layout(prepared, safetyWidth, this.options.lineHeight);
        return Math.ceil(Math.max(this.options.lineHeight, result.height)); // [FIX] 強制進位確保不重疊
    }

    recalculateHeights() {
        const width = this.container.clientWidth - 20;
        this.totalHeight = 0;
        this.cumulativeHeights = []; // [NEW] 重置累加高度
        this.itemHeights = this.logs.map((log) => {
            const h = this.measureHeight(log.message, width);
            log.height = h;
            this.totalHeight += h;
            this.cumulativeHeights.push(this.totalHeight); // [NEW] 重新構建累加高度
            return h;
        });
        this.scroller.style.height = `${this.totalHeight + this.paddingY}px`;
    }

    handleScroll() {
        if (!this.isProgrammaticScroll) {
            this.lastUserScrollAt = Date.now();
            this.suspendAutoScroll = true;
            const { scrollTop, scrollHeight, clientHeight } = this.container;
            // 判斷是否鎖定在底部 (保留 30px 的誤觸空間)
            const gap = scrollHeight - (scrollTop + clientHeight);
            const locked = gap <= this.lockThreshold;
            if (this.isLockedToBottom !== locked) {
                this.isLockedToBottom = locked;
                this.triggerUpdate();
            }
        }
        if (this.suspendAutoScroll && Date.now() - this.lastUserScrollAt > this.userScrollGraceMs) {
            this.suspendAutoScroll = false;
        }
        this.render();
    }

    triggerUpdate() {
        if (this.onUpdate) {
            this.onUpdate({
                total: this.logs.length,
                rendered: this.lastRenderedCount || 0,
                isLocked: this.isLockedToBottom,
            });
        }
    }

    /**
     * [NEW] 使用二分搜尋在累加高度數組中尋找對應偏移量的索引
     * 複雜度: O(log N)
     */
    findIndexAtOffset(offset) {
        let low = 0;
        let high = this.cumulativeHeights.length - 1;
        while (low <= high) {
            const mid = (low + high) >>> 1;
            if (this.cumulativeHeights[mid] <= offset) {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }
        return low;
    }

    render() {
        if (this.logs.length === 0) return;

        const scrollTop = this.container.scrollTop;
        const viewHeight = this.container.clientHeight;

        // [OPTIMIZED] 使用二分搜尋取代線性遍歷 (O(log N))
        let startIndex = this.findIndexAtOffset(scrollTop);
        let endIndex = this.findIndexAtOffset(scrollTop + viewHeight);

        // 套用緩衝區渲染
        startIndex = Math.max(0, startIndex - this.options.buffer);
        endIndex = Math.min(this.logs.length - 1, endIndex + this.options.buffer);

        // [OPTIMIZED] O(1) 取得精確位移
        const exactTop = startIndex > 0 ? this.cumulativeHeights[startIndex - 1] : 0;

        this.viewport.style.transform = `translateY(${exactTop}px)`;
        this.lastRenderedCount = endIndex - startIndex + 1; // [FIX] 使用正確的變數名稱
        this.renderSlice(startIndex, endIndex);
        this.syncBottomIfNeeded();
        this.triggerUpdate(); // [NEW] 觸發更新
    }

    renderSlice(start, end) {
        const fragment = document.createDocumentFragment();
        for (let i = start; i <= end; i++) {
            const log = this.logs[i];
            const div = document.createElement('div');
            div.className = `log-line log-${log.level}`;
            div.style.minHeight = `${log.height}px`; // [FIX] 使用 min-height

            const timeSpan = document.createElement('span');
            timeSpan.className = 'log-time';
            timeSpan.textContent = log.timeStr ? `[${log.timeStr}] ` : '';

            const msgSpan = document.createElement('span');
            msgSpan.className = 'log-msg';

            // [NEW] 解析標籤並轉換為帶有類別的 Span
            const parts = log.message.split(/(<dir>.*?<\/dir>|<file>.*?<\/file>)/g);
            parts.forEach((part) => {
                if (part.startsWith('<dir>')) {
                    const s = document.createElement('span');
                    s.className = 'log-dir';
                    s.textContent = part.replace(/<dir>|<\/dir>/g, '');
                    msgSpan.appendChild(s);
                } else if (part.startsWith('<file>')) {
                    const s = document.createElement('span');
                    s.className = 'log-file';
                    s.textContent = part.replace(/<file>|<\/file>/g, '');
                    msgSpan.appendChild(s);
                } else if (part) {
                    msgSpan.appendChild(document.createTextNode(part));
                }
            });

            div.appendChild(timeSpan);
            div.appendChild(msgSpan);
            fragment.appendChild(div);
        }
        this.viewport.innerHTML = '';
        this.viewport.appendChild(fragment);
    }
}
