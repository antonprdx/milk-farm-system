import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import Modal from '$lib/components/ui/Modal.svelte';

describe('Modal', () => {
	it('does not render when closed', () => {
		const { container } = render(Modal, {
			props: {
				open: false,
				title: 'Test',
				// @ts-expect-error snippet children
				children: () => {},
			},
		});
		expect(container.querySelector('[role="dialog"]')).toBeNull();
	});

	it('renders when open with title', () => {
		const { getByText, container } = render(Modal, {
			props: {
				open: true,
				title: 'Test Modal',
				// @ts-expect-error snippet children
				children: () => {},
			},
		});
		expect(getByText('Test Modal')).toBeTruthy();
		expect(container.querySelector('[aria-modal="true"]')).toBeTruthy();
	});
});
