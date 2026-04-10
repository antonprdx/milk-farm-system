import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import Pagination from '$lib/components/ui/Pagination.svelte';

describe('Pagination', () => {
	it('renders nothing when total is 0', () => {
		const { container } = render(Pagination, { props: { page: 1, total: 0, perPage: 20 } });
		expect(container.querySelector('button')).toBeNull();
	});

	it('renders when total > 0', () => {
		const { getByText } = render(Pagination, { props: { page: 1, total: 50, perPage: 20 } });
		expect(getByText('Всего: 50')).toBeTruthy();
		expect(getByText('1 / 3')).toBeTruthy();
	});

	it('disables prev on page 1', () => {
		const { getByText } = render(Pagination, { props: { page: 1, total: 50, perPage: 20 } });
		expect((getByText('Назад') as HTMLButtonElement).disabled).toBe(true);
		expect((getByText('Вперёд') as HTMLButtonElement).disabled).toBe(false);
	});

	it('disables next on last page', () => {
		const { getByText } = render(Pagination, { props: { page: 3, total: 50, perPage: 20 } });
		expect((getByText('Назад') as HTMLButtonElement).disabled).toBe(false);
		expect((getByText('Вперёд') as HTMLButtonElement).disabled).toBe(true);
	});

	it('shows correct page count', () => {
		const { getByText } = render(Pagination, { props: { page: 2, total: 45, perPage: 20 } });
		expect(getByText('2 / 3')).toBeTruthy();
	});
});
