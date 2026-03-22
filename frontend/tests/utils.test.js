import { describe, expect, it } from 'vitest';
import { escapeHtml } from '../modules/utils.js';

describe('utils.js', () => {
    describe('escapeHtml', () => {
        it('should escape HTML tags correctly', () => {
            const input = '<div>Test</div>';
            const output = '&lt;div&gt;Test&lt;/div&gt;';
            expect(escapeHtml(input)).toBe(output);
        });

        it('should escape quotes correctly', () => {
            const input = '"Safe" & \'Sound\'';
            const output = '&quot;Safe&quot; &amp; &#39;Sound&#39;';
            expect(escapeHtml(input)).toBe(output);
        });

        it('should handle empty or null values', () => {
            expect(escapeHtml('')).toBe('');
            expect(escapeHtml(null)).toBe('');
            expect(escapeHtml(undefined)).toBe('');
        });

        it('should neutralize scripts', () => {
            const input = '<script>alert("xss")</script>';
            const output = '&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;';
            expect(escapeHtml(input)).toBe(output);
        });
    });
});
