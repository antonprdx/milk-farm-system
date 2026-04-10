import { describe, it, expect } from 'vitest';
import { rules } from '$lib/utils/validators';

describe('rules helpers', () => {
	it('required() returns error for empty', () => {
		const r = rules.required();
		expect(r.required).toBe(true);
	});

	it('email() matches valid emails', () => {
		const r = rules.email();
		expect(r.pattern).toBeTruthy();
		expect(r.pattern!.test('test@example.com')).toBe(true);
		expect(r.pattern!.test('invalid')).toBe(false);
	});

	it('minLength() sets correct length', () => {
		const r = rules.minLength(5);
		expect(r.minLength).toBe(5);
	});

	it('percentage() has min 0 max 100', () => {
		const r = rules.percentage();
		expect(r.min).toBe(0);
		expect(r.max).toBe(100);
	});

	it('positive() has min 0.001', () => {
		const r = rules.positive();
		expect(r.min).toBe(0.001);
	});

	it('nonNegative() has min 0', () => {
		const r = rules.nonNegative();
		expect(r.min).toBe(0);
	});
});
