export default [
    {
        files: ["frontend/**/*.js"],
        languageOptions: {
            ecmaVersion: "latest",
            sourceType: "module",
            globals: {
                window: "readonly",
                document: "readonly",
                console: "readonly",
                setTimeout: "readonly",
                setInterval: 'readonly',
                clearTimeout: 'readonly',
                clearInterval: 'readonly',
                alert: 'readonly',
                confirm: 'readonly',
                ResizeObserver: 'readonly',
                Intl: 'readonly',
                performance: "readonly",
                documentFragment: "readonly",
                navigator: "readonly",
                localStorage: "readonly",
                sessionStorage: "readonly",
                JSON: "readonly",
                Map: "readonly",
                Set: "readonly",
                URL: "readonly",
                fetch: "readonly",
                location: "readonly",
                import: "readonly", // for dynamic imports
                HTMLElement: "readonly",
                HTMLScriptElement: "readonly",
                Event: "readonly",
                CustomEvent: "readonly",
            }
        },
        rules: {
            "no-unused-vars": "warn",
            "no-undef": "error"
        }
    }
];
