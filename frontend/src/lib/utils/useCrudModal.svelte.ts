import { toasts } from '$lib/stores/toast';

export function useCrudModal() {
	let showModal = $state(false);
	let modalMode = $state<'create' | 'edit'>('create');
	let modalLoading = $state(false);
	let modalError = $state('');
	let editingId = $state(0);

	let showDelete = $state(false);
	let deleteLoading = $state(false);
	let deleteId = $state(0);

	function openCreate() {
		modalMode = 'create';
		modalError = '';
		showModal = true;
	}

	function openEdit(id: number) {
		modalMode = 'edit';
		editingId = id;
		modalError = '';
		showModal = true;
	}

	function close() {
		showModal = false;
	}

	function confirmDelete(id: number) {
		deleteId = id;
		showDelete = true;
	}

	async function submit(fn: () => Promise<unknown>, successMsg: string, reload: () => void) {
		modalError = '';
		try {
			modalLoading = true;
			await fn();
			showModal = false;
			toasts.success(successMsg);
			reload();
		} catch (e) {
			modalError = e instanceof Error ? e.message : 'Ошибка';
		} finally {
			modalLoading = false;
		}
	}

	async function remove(
		fn: () => Promise<unknown>,
		reload: () => void,
		errorMsg?: (msg: string) => void,
	) {
		try {
			deleteLoading = true;
			await fn();
			showDelete = false;
			toasts.success('Запись удалена');
			reload();
		} catch (e) {
			const msg = e instanceof Error ? e.message : 'Ошибка удаления';
			if (errorMsg) {
				errorMsg(msg);
			} else {
				toasts.error(msg);
			}
			showDelete = false;
		} finally {
			deleteLoading = false;
		}
	}

	return {
		get showModal() {
			return showModal;
		},
		set showModal(v: boolean) {
			showModal = v;
		},
		get modalMode() {
			return modalMode;
		},
		get modalLoading() {
			return modalLoading;
		},
		get modalError() {
			return modalError;
		},
		set modalError(v: string) {
			modalError = v;
		},
		get editingId() {
			return editingId;
		},
		get showDelete() {
			return showDelete;
		},
		get deleteLoading() {
			return deleteLoading;
		},
		get deleteId() {
			return deleteId;
		},
		openCreate,
		openEdit,
		close,
		confirmDelete,
		closeDelete: () => {
			showDelete = false;
		},
		submit,
		remove,
	};
}
