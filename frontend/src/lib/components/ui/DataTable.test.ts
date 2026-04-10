import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import DataTable from '$lib/components/ui/DataTable.svelte';

const columns = [
	{ key: 'name', label: 'Имя' },
	{ key: 'value', label: 'Значение' },
];

describe('DataTable', () => {
	it('shows 5 loading skeleton rows when loading is true', () => {
		const { container } = render(DataTable, {
			// @ts-expect-error -- Svelte 5 snippet props not fully typed for tests
			props: { columns, loading: true, children: () => '' },
		});
		const rows = container.querySelectorAll('tbody tr');
		expect(rows.length).toBe(5);
		rows.forEach((row) => {
			expect(row.querySelector('.animate-pulse')).toBeTruthy();
		});
	});
});
