import { describe, it, expect, vi } from 'vitest';
import { useCrudModal } from '$lib/utils/useCrudModal.svelte';

describe('useCrudModal', () => {
	it('starts with showModal false', () => {
		const modal = useCrudModal();
		expect(modal.showModal).toBe(false);
	});

	it('starts with showDelete false', () => {
		const modal = useCrudModal();
		expect(modal.showDelete).toBe(false);
	});

	it('openCreate sets showModal and mode', () => {
		const modal = useCrudModal();
		modal.openCreate();
		expect(modal.showModal).toBe(true);
		expect(modal.modalMode).toBe('create');
	});

	it('openEdit sets showModal, mode, and editingId', () => {
		const modal = useCrudModal();
		modal.openEdit(42);
		expect(modal.showModal).toBe(true);
		expect(modal.modalMode).toBe('edit');
		expect(modal.editingId).toBe(42);
	});

	it('close hides modal', () => {
		const modal = useCrudModal();
		modal.openCreate();
		modal.close();
		expect(modal.showModal).toBe(false);
	});

	it('confirmDelete sets showDelete and deleteId', () => {
		const modal = useCrudModal();
		modal.confirmDelete(7);
		expect(modal.showDelete).toBe(true);
		expect(modal.deleteId).toBe(7);
	});

	it('closeDelete hides delete dialog', () => {
		const modal = useCrudModal();
		modal.confirmDelete(1);
		modal.closeDelete();
		expect(modal.showDelete).toBe(false);
	});

	it('submit calls fn and closes modal on success', async () => {
		const modal = useCrudModal();
		modal.openCreate();
		const fn = vi.fn().mockResolvedValue(undefined);
		const reload = vi.fn();
		await modal.submit(fn, 'Done', reload);
		expect(fn).toHaveBeenCalledOnce();
		expect(modal.showModal).toBe(false);
		expect(reload).toHaveBeenCalledOnce();
	});

	it('submit sets modalError on failure', async () => {
		const modal = useCrudModal();
		modal.openCreate();
		const fn = vi.fn().mockRejectedValue(new Error('fail'));
		await modal.submit(fn, 'Done', vi.fn());
		expect(modal.showModal).toBe(true);
		expect(modal.modalError).toBe('fail');
	});

	it('remove calls fn and closes dialog on success', async () => {
		const modal = useCrudModal();
		modal.confirmDelete(5);
		const fn = vi.fn().mockResolvedValue(undefined);
		const reload = vi.fn();
		await modal.remove(fn, reload);
		expect(fn).toHaveBeenCalledOnce();
		expect(modal.showDelete).toBe(false);
		expect(reload).toHaveBeenCalledOnce();
	});

	it('remove calls errorMsg callback on failure', async () => {
		const modal = useCrudModal();
		modal.confirmDelete(5);
		const fn = vi.fn().mockRejectedValue(new Error('boom'));
		const errorCb = vi.fn();
		await modal.remove(fn, vi.fn(), errorCb);
		expect(errorCb).toHaveBeenCalledWith('boom');
		expect(modal.showDelete).toBe(false);
	});

	it('clears modalError on openCreate', () => {
		const modal = useCrudModal();
		modal.modalError = 'old error';
		modal.openCreate();
		expect(modal.modalError).toBe('');
	});

	it('clears modalError on openEdit', () => {
		const modal = useCrudModal();
		modal.modalError = 'old error';
		modal.openEdit(1);
		expect(modal.modalError).toBe('');
	});
});
