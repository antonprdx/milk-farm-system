<script lang="ts">
	import { onMount } from 'svelte';
	import {
		listUsers,
		createUser,
		changePassword,
		deleteUser,
		updateUserRole,
		type UserItem,
	} from '$lib/api/settings';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import { toasts } from '$lib/stores/toast';
	import { auth } from '$lib/stores/auth';
	import { useFormValidation } from '$lib/utils/useFormValidation.svelte';
	import { rules } from '$lib/utils/validators';

	let loading = $state(true);
	let error = $state('');
	let users = $state<UserItem[]>([]);

	let showCreate = $state(false);
	let createLoading = $state(false);
	let createError = $state('');
	let newUsername = $state('');
	let newPassword = $state('');

	let showPassword = $state(false);
	let pwdLoading = $state(false);
	let pwdError = $state('');
	let oldPwd = $state('');
	let newPwd = $state('');

	let showDelete = $state(false);
	let deleteLoading = $state(false);
	let deleteUserId = $state<number | null>(null);
	let deleteUsername = $state('');

	const createV = useFormValidation();
	const pwdV = useFormValidation();

	async function load() {
		try {
			loading = true;
			error = '';
			users = (await listUsers()).data;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка загрузки';
		} finally {
			loading = false;
		}
	}

	async function handleCreate(e: Event) {
		e.preventDefault();
		createError = '';
		if (
			!createV.validateAll({
				newUsername: { value: newUsername, rules: [rules.required(), rules.minLength(3)] },
				newPassword: { value: newPassword, rules: [rules.required(), rules.minLength(6)] },
			})
		)
			return;
		try {
			createLoading = true;
			await createUser({ username: newUsername, password: newPassword });
			showCreate = false;
			newUsername = '';
			newPassword = '';
			toasts.success('Пользователь создан');
			load();
		} catch (e) {
			createError = e instanceof Error ? e.message : 'Ошибка создания';
		} finally {
			createLoading = false;
		}
	}

	async function handleChangePassword(e: Event) {
		e.preventDefault();
		pwdError = '';
		if (
			!pwdV.validateAll({
				oldPwd: { value: oldPwd, rules: [rules.required()] },
				newPwd: { value: newPwd, rules: [rules.required(), rules.minLength(6)] },
			})
		)
			return;
		try {
			pwdLoading = true;
			await changePassword({ old_password: oldPwd, new_password: newPwd });
			auth.clearMustChangePassword();
			showPassword = false;
			oldPwd = '';
			newPwd = '';
			toasts.success('Пароль изменён');
		} catch (e) {
			pwdError = e instanceof Error ? e.message : 'Ошибка';
		} finally {
			pwdLoading = false;
		}
	}

	onMount(load);

	async function handleDelete() {
		if (deleteUserId === null) return;
		try {
			deleteLoading = true;
			await deleteUser(deleteUserId);
			showDelete = false;
			toasts.success('Пользователь удалён');
			load();
		} catch (e) {
			toasts.error(e instanceof Error ? e.message : 'Ошибка удаления');
		} finally {
			deleteLoading = false;
		}
	}

	async function handleRoleChange(userId: number, newRole: string) {
		try {
			await updateUserRole(userId, newRole);
			toasts.success('Роль изменена');
			load();
		} catch (e) {
			toasts.error(e instanceof Error ? e.message : 'Ошибка');
		}
	}
</script>

<svelte:head>
	<title>Настройки — Молочная ферма</title>
</svelte:head>

<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100 mb-6">Настройки</h1>

<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
	<div>
		<div class="flex items-center justify-between mb-4">
			<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300">Пользователи</h2>
			<button
				onclick={() => {
					showCreate = true;
					newUsername = '';
					newPassword = '';
					createError = '';
					createV.clear();
				}}
				class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
			>
				+ Добавить
			</button>
		</div>

		<ErrorAlert message={error} />

		<DataTable
			columns={[
				{ key: 'id', label: 'ID' },
				{ key: 'username', label: 'Имя' },
				{ key: 'role', label: 'Роль' },
				{ key: 'created_at', label: 'Создан' },
			]}
			{loading}
		>
			{#if users.length === 0}
				<tr
					><td colspan="4" class="px-4 py-8 text-center text-slate-400 dark:text-slate-500"
						>Нет пользователей</td
					></tr
				>
			{:else}
				{#each users as u (u.id)}
					<tr
						class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
					>
						<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{u.id}</td>
						<td class="px-4 py-3 font-medium">{u.username}</td>
						<td class="px-4 py-3">
							<select
								value={u.role}
								onchange={() => handleRoleChange(u.id, u.role === 'admin' ? 'user' : 'admin')}
								class="text-xs border border-slate-300 dark:border-slate-600 rounded px-1 py-0.5"
							>
								<option value="admin">admin</option>
								<option value="user">user</option>
							</select>
						</td>
						<td class="px-4 py-3 text-slate-500 dark:text-slate-400"
							>{new Date(u.created_at).toLocaleDateString('ru-RU')}</td
						>
						<td class="px-4 py-3">
							<button
								onclick={() => {
									deleteUserId = u.id;
									deleteUsername = u.username;
									showDelete = true;
								}}
								class="text-red-500 hover:text-red-700 text-xs cursor-pointer">Удалить</button
							>
						</td>
					</tr>
				{/each}
			{/if}
		</DataTable>
	</div>

	<div>
		<div class="flex items-center justify-between mb-4">
			<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300">Сменить пароль</h2>
			<button
				onclick={() => {
					showPassword = true;
					oldPwd = '';
					newPwd = '';
					pwdError = '';
					pwdV.clear();
				}}
				class="px-4 py-2 bg-slate-100 dark:bg-slate-900 hover:bg-slate-200 dark:bg-slate-700 text-slate-700 dark:text-slate-300 text-sm font-medium rounded-lg transition-colors cursor-pointer"
			>
				Сменить пароль
			</button>
		</div>

		{#if users.length > 0}
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
			>
				<p class="text-sm text-slate-600 dark:text-slate-400">
					Зарегистрировано пользователей: <span
						class="font-medium text-slate-800 dark:text-slate-100">{users.length}</span
					>
				</p>
			</div>
		{/if}
	</div>
</div>

<Modal open={showCreate} title="Новый пользователь" onclose={() => (showCreate = false)}>
	<ErrorAlert message={createError} />
	<form onsubmit={handleCreate} class="space-y-4">
		<FormField
			id="c-user"
			label="Имя пользователя"
			bind:value={newUsername}
			required
			error={createV.getError('newUsername')}
		/>
		<FormField
			id="c-pass"
			label="Пароль"
			type="password"
			bind:value={newPassword}
			required
			error={createV.getError('newPassword')}
		/>
		<div class="flex gap-3 justify-end pt-2">
			<button
				type="button"
				onclick={() => (showCreate = false)}
				class="px-4 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:bg-slate-800/50 cursor-pointer"
				>Отмена</button
			>
			<button
				type="submit"
				disabled={createLoading}
				class="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg disabled:opacity-50 cursor-pointer"
			>
				{createLoading ? 'Создание...' : 'Создать'}
			</button>
		</div>
	</form>
</Modal>

<Modal open={showPassword} title="Сменить пароль" onclose={() => (showPassword = false)}>
	<ErrorAlert message={pwdError} />
	<form onsubmit={handleChangePassword} class="space-y-4">
		<FormField
			id="p-old"
			label="Текущий пароль"
			type="password"
			bind:value={oldPwd}
			required
			error={pwdV.getError('oldPwd')}
		/>
		<FormField
			id="p-new"
			label="Новый пароль"
			type="password"
			bind:value={newPwd}
			required
			error={pwdV.getError('newPwd')}
		/>
		<div class="flex gap-3 justify-end pt-2">
			<button
				type="button"
				onclick={() => (showPassword = false)}
				class="px-4 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:bg-slate-800/50 cursor-pointer"
				>Отмена</button
			>
			<button
				type="submit"
				disabled={pwdLoading}
				class="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg disabled:opacity-50 cursor-pointer"
			>
				{pwdLoading ? 'Сохранение...' : 'Сменить'}
			</button>
		</div>
	</form>
</Modal>

<ConfirmDialog
	open={showDelete}
	title="Удалить пользователя"
	message="Удалить пользователя «{deleteUsername}»? Это действие нельзя отменить."
	confirmText="Удалить"
	loading={deleteLoading}
	onconfirm={handleDelete}
	oncancel={() => (showDelete = false)}
/>
