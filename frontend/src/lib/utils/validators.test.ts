import { describe, it, expect } from 'vitest';
import { validate, rules } from '$lib/utils/validators';

describe('validate', () => {
	it('returns undefined for valid required value', () => {
		expect(validate('hello', [rules.required()])).toBeUndefined();
	});

	it('returns error for empty required value', () => {
		expect(validate('', [rules.required()])).toBe('Поле обязательно для заполнения');
	});

	it('returns error for whitespace-only required value', () => {
		expect(validate('   ', [rules.required()])).toBe('Поле обязательно для заполнения');
	});

	it('returns error for value shorter than minLength', () => {
		expect(validate('ab', [rules.minLength(3)])).toBe('Минимум 3 символов');
	});

	it('passes for value meeting minLength', () => {
		expect(validate('abc', [rules.minLength(3)])).toBeUndefined();
	});

	it('returns error for value exceeding maxLength', () => {
		expect(validate('abcde', [rules.maxLength(3)])).toBe('Максимум 3 символов');
	});

	it('returns error for number below min', () => {
		expect(validate(-1, [rules.min(0)])).toBe('Минимальное значение: 0');
	});

	it('returns error for number above max', () => {
		expect(validate(101, [rules.max(100)])).toBe('Максимальное значение: 100');
	});

	it('validates percentage range', () => {
		expect(validate(50, [rules.percentage()])).toBeUndefined();
		expect(validate(-1, [rules.percentage()])).toBeTruthy();
		expect(validate(101, [rules.percentage()])).toBeTruthy();
	});

	it('validates positive numbers', () => {
		expect(validate(1, [rules.positive()])).toBeUndefined();
		expect(validate(0, [rules.positive()])).toBeTruthy();
		expect(validate(-5, [rules.positive()])).toBeTruthy();
	});

	it('validates email pattern', () => {
		expect(validate('test@example.com', [rules.email()])).toBeUndefined();
		expect(validate('not-email', [rules.email()])).toBeTruthy();
	});

	it('validates date not in future', () => {
		expect(validate('2020-01-01', [rules.dateNotFuture()])).toBeUndefined();
	});

	it('runs custom validator', () => {
		const custom = {
			custom: (v: string | number | undefined) => (v === 'bad' ? 'Bad value' : undefined),
		};
		expect(validate('bad', [custom])).toBe('Bad value');
		expect(validate('good', [custom])).toBeUndefined();
	});

	it('returns first failing rule', () => {
		const result = validate('', [rules.minLength(5), rules.required()]);
		expect(result).toBeTruthy();
	});
});
