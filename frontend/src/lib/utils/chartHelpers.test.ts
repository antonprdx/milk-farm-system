import { describe, it, expect } from 'vitest';
import { themeColors, defaultTooltip, defaultScales, dsColors } from '$lib/utils/chartHelpers';

describe('themeColors', () => {
	it('returns dark colors for dark mode', () => {
		const c = themeColors(true);
		expect(c.gridColor).toContain('148');
		expect(c.textColor).toBe('#94a3b8');
	});

	it('returns light colors for light mode', () => {
		const c = themeColors(false);
		expect(c.gridColor).toContain('203');
		expect(c.textColor).toBe('#64748b');
	});
});

describe('defaultTooltip', () => {
	it('returns dark tooltip for dark mode', () => {
		const t = defaultTooltip(true);
		expect(t.backgroundColor).toBe('#1e293b');
		expect(t.borderWidth).toBe(1);
	});

	it('includes callbacks when provided', () => {
		const t = defaultTooltip(false, { label: () => 'test' });
		expect(t.callbacks).toBeDefined();
	});

	it('omits callbacks when not provided', () => {
		const t = defaultTooltip(false);
		expect((t as Record<string, unknown>).callbacks).toBeUndefined();
	});
});

describe('defaultScales', () => {
	it('returns x and y scales', () => {
		const s = defaultScales(false);
		expect(s.x).toBeDefined();
		expect(s.y).toBeDefined();
		expect(s.y.beginAtZero).toBe(true);
	});

	it('includes y callback when provided', () => {
		const cb = (v: string | number) => String(v);
		const s = defaultScales(false, cb);
		expect((s.y.ticks as Record<string, unknown>).callback).toBeDefined();
	});
});

describe('dsColors', () => {
	it('returns blue colors by default', () => {
		const c = dsColors(false);
		expect(c.border).toContain('37');
		expect(c.bg).toContain('246');
		expect(c.point).toBeDefined();
	});

	it('returns red colors for red hue', () => {
		const c = dsColors(true, 'red');
		expect(c.border).toContain('248');
	});

	it('returns green colors for green hue', () => {
		const c = dsColors(false, 'green');
		expect(c.border).toContain('150');
	});
});
