import { defineConfig } from 'vitest/config';

export default defineConfig({
    test: {
        environment: 'happy-dom',
        globals: true,
        include: ['tests/frontend/**/*.test.js'],
        setupFiles: ['./tests/frontend/setup.js'],

        coverage: {
            provider: 'v8',
            reporter: ['text', 'json', 'lcov'],
            thresholds: {
                lines: 80,
                branches: 60,
                functions: 70,
                statements: 80,
            },
        },
    },
});
