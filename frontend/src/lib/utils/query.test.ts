import { describe, it, expect } from 'vitest';
import { buildQuery } from '$lib/utils/query';

describe('buildQuery', () => {
	it('returns empty string for empty object', () => {
		expect(buildQuery({})).toBe('');
	});

	it('returns query string for simple values', () => {
		const result = buildQuery({ page: 1, per_page: 20 });
		expect(result).toContain('page=1');
		expect(result).toContain('per_page=20');
	});

	it('omits undefined values', () => {
		const result = buildQuery({ page: 1, animal_id: undefined });
		expect(result).toContain('page=1');
		expect(result).not.toContain('animal_id');
	});

	it('omits empty string values', () => {
		const result = buildQuery({ page: 1, search: '' });
		expect(result).toContain('page=1');
		expect(result).not.toContain('search');
	});

	it('handles zero values correctly', () => {
		const result = buildQuery({ offset: 0 });
		expect(result).toContain('offset=0');
	});

	it('handles string values', () => {
		const result = buildQuery({ from_date: '2025-01-01', till_date: '2025-12-31' });
		expect(result).toContain('from_date=2025-01-01');
		expect(result).toContain('till_date=2025-12-31');
	});
});
