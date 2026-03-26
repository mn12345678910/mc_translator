import { defineConfig } from 'vitest/config';
import path from 'path';

export default defineConfig({
    resolve: {
        alias: {
            '@tauri-apps/api/tauri': path.resolve(__dirname, 'tests/frontend/tauri_mock.js'),
        },
    },
    test: {
        environment: 'happy-dom', // 模擬瀏覽器環境
        globals: true,
        include: ['tests/frontend/**/*.test.js'],
        setupFiles: ['./tests/frontend/setup.js'],

        coverage: {
            provider: 'v8',
            reporter: ['text', 'json', 'lcov'],
        },

    },
});
