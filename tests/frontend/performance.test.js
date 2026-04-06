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

describe('VirtualLogViewer Performance Benchmark', () => {
    let container;
    let viewer;

    beforeEach(() => {
        document.body.innerHTML = '<div id="log-container" style="height: 400px; width: 600px; overflow: auto;"></div>';
        container = document.getElementById('log-container');
        // Mock clientHeight and clientWidth
        Object.defineProperty(container, 'clientHeight', { value: 400, configurable: true });
        Object.defineProperty(container, 'clientWidth', { value: 600, configurable: true });

        // Mock measureHeight to avoid canvas dependency in tests
        vi.spyOn(VirtualLogViewer.prototype, 'measureHeight').mockReturnValue(20);

        viewer = new VirtualLogViewer('log-container');
    });

    const measureRender = (count) => {
        // Pre-fill logs and itemHeights to avoid O(N^2) during setup
        const width = container.clientWidth - 20;
        for (let i = 0; i < count; i++) {
            const message = `Test log entry #${i} with some content.`;
            const height = viewer.measureHeight(message, width);
            viewer.logs.push({ message, level: 'info', timeStr: '12:00:00', height });
            viewer.itemHeights.push(height);
            viewer.totalHeight += height;
        }
        viewer.scroller.style.height = `${viewer.totalHeight + viewer.paddingY}px`;

        // Measure single render at start
        const start1 = performance.now();
        viewer.render();
        const end1 = performance.now();

        // Measure single render at end (Worst case for exactTop calculation)
        container.scrollTop = viewer.totalHeight;
        const start2 = performance.now();
        viewer.render();
        const end2 = performance.now();

        return { startRender: end1 - start1, endRender: end2 - start2 };
    };

    it('benchmarks with 10,000 logs', () => {
        const res = measureRender(10000);
        console.log(
            `BENCHMARK_RESULT: [10k] Start: ${res.startRender.toFixed(2)}ms, End: ${res.endRender.toFixed(2)}ms`
        );
        expect(res.startRender).toBeLessThan(100);
        expect(res.endRender).toBeLessThan(100);
        viewer.logs = [];
        viewer.itemHeights = []; // Clear for memory
    });

    it('benchmarks with 30,000 logs', () => {
        const res = measureRender(30000);
        console.log(
            `BENCHMARK_RESULT: [30k] Start: ${res.startRender.toFixed(2)}ms, End: ${res.endRender.toFixed(2)}ms`
        );
        expect(res.startRender).toBeLessThan(300);
        expect(res.endRender).toBeLessThan(300);
        viewer.logs = [];
        viewer.itemHeights = [];
    });

    it('benchmarks with 100,000 logs', () => {
        const res = measureRender(100000);
        console.log(
            `BENCHMARK_RESULT: [100k] Start: ${res.startRender.toFixed(2)}ms, End: ${res.endRender.toFixed(2)}ms`
        );
        expect(res.startRender).toBeLessThan(1000);
        expect(res.endRender).toBeLessThan(1000);
        viewer.logs = [];
        viewer.itemHeights = [];
        viewer.cumulativeHeights = [];
    });
});
