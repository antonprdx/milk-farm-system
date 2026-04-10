import { describe, it, expect } from 'vitest';
import { fmtNum, formatDate, formatDatetime } from '$lib/utils/format';

describe('fmtNum', () => {
	it('formats a number with default 1 decimal', () => {
		expect(fmtNum(3.14)).toBe('3.1');
	});

	it('formats a number with specified decimals', () => {
		expect(fmtNum(3.14159, 3)).toBe('3.142');
	});

	it('returns em dash for null', () => {
		expect(fmtNum(null)).toBe('—');
	});

	it('returns em dash for undefined', () => {
		expect(fmtNum(undefined)).toBe('—');
	});

	it('formats zero', () => {
		expect(fmtNum(0)).toBe('0.0');
	});

	it('formats negative number', () => {
		expect(fmtNum(-5.5)).toBe('-5.5');
	});
});

describe('formatDate', () => {
	it('returns date string unchanged', () => {
		expect(formatDate('2025-01-15')).toBe('2025-01-15');
	});
});

describe('formatDatetime', () => {
	it('formats ISO datetime to locale string', () => {
		const result = formatDatetime('2025-01-15T10:30:00Z');
		expect(result).toBeTruthy();
		expect(typeof result).toBe('string');
	});
});
