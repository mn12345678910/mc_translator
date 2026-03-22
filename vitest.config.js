import { defineConfig } from 'vitest/config';

export default defineConfig({
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
