<script lang="ts">
	import { onMount } from 'svelte';
	import {
		listUsers,
		createUser,
		changePassword,
		deleteUser,
		updateUserRole,
		getSystemInfo,
		getJwtTtl,
		updateJwtTtl,
		getAlertThresholds,
		updateAlertThresholds,
		getBackupUrl,
		type UserItem,
		type SystemInfo,
		type JwtTtlSettings,
		type AlertThresholds,
	} from '$lib/api/settings';
	import {
		getLelyStatus,
		triggerLelySync,
		getLelyConfig,
		type LelySyncStatus,
		type LelyConfigResponse,
	} from '$lib/api/lely';
	import { preferences } from '$lib/stores/preferences';
	import { auth } from '$lib/stores/auth';
	import { theme } from '$lib/stores/theme';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import { toasts } from '$lib/stores/toast';
	import { useFormValidation } from '$lib/utils/useFormValidation.svelte';
	import { rules } from '$lib/utils/validators';
	import {
		Sun,
		Moon,
		Monitor,
		Save,
		Download,
		Shield,
		Bell,
		Database,
		Users,
		Settings,
		BarChart3,
		RefreshCw,
		Plug,
	} from 'lucide-svelte';

	let dataTable: DataTable | undefined = $state();
	let activeTab = $state('profile');
	let loading = $state(true);
	let error = $state('');
	let users = $state<UserItem[]>([]);
	let isAdmin = $derived($auth.role === 'admin');

	// Profile / Password
	let showPassword = $state(false);
	let pwdLoading = $state(false);
	let pwdError = $state('');
	let oldPwd = $state('');
	let newPwd = $state('');
	const pwdV = useFormValidation();

	// Create user
	let showCreate = $state(false);
	let createLoading = $state(false);
	let createError = $state('');
	let newUsername = $state('');
	let newPassword = $state('');
	const createV = useFormValidation();

	// Delete user
	let showDelete = $state(false);
	let deleteLoading = $state(false);
	let deleteUserId = $state<number | null>(null);
	let deleteUsername = $state('');

	let showRoleConfirm = $state(false);
	let pendingRoleUserId = $state(0);
	let pendingRoleNew = $state('');

	// Preferences
	let prefsTheme = $state($preferences.theme);
	let prefsPageSize = $state($preferences.page_size);
	let prefsCompact = $state($preferences.compact_view);
	let prefsLang = $state($preferences.language);

	// System info
	let sysInfo = $state<SystemInfo | null>(null);

	// JWT TTL
	let jwtTtl = $state<JwtTtlSettings | null>(null);
	let jwtAccessTtl = $state('');
	let jwtRefreshTtl = $state('');
	let jwtSaving = $state(false);

	// Alert thresholds
	let thresholds = $state<AlertThresholds | null>(null);
	let threshMinMilk = $state('');
	let threshMaxScc = $state('');
	let threshDaysCalving = $state('');
	let threshActivityDrop = $state('');
	let threshSaving = $state(false);

	// Lely integration
	let lelyStatus = $state<LelySyncStatus[]>([]);
	let lelyConfig = $state<LelyConfigResponse | null>(null);
	let lelySyncing = $state(false);
	let lelyLoading = $state(false);

	let tabs = $derived([
		{ key: 'profile', label: 'Профиль' },
		{ key: 'interface', label: 'Интерфейс' },
		...(isAdmin
			? [
					{ key: 'users', label: 'Пользователи' },
					{ key: 'system', label: 'Система' },
					{ key: 'security', label: 'Безопасность' },
					{ key: 'alerts', label: 'Уведомления' },
					{ key: 'lely', label: 'Интеграция' },
				]
			: []),
	]);

	async function load() {
		try {
			loading = true;
			error = '';
			users = (await listUsers()).data;
			dataTable?.setHasRows(users.length > 0);
			if (isAdmin) {
				getSystemInfo()
					.then((i) => (sysInfo = i))
					.catch((e) => console.warn('Failed to load system info', e));
				getJwtTtl()
					.then((t) => {
						jwtTtl = t;
						jwtAccessTtl = t.jwt_access_ttl_secs.toString();
						jwtRefreshTtl = t.jwt_refresh_ttl_secs.toString();
					})
					.catch((e) => console.warn('Failed to load JWT TTL settings', e));
				getAlertThresholds()
					.then((t) => {
						thresholds = t;
						threshMinMilk = t.alert_min_milk.toString();
						threshMaxScc = t.alert_max_scc.toString();
						threshDaysCalving = t.alert_days_before_calving.toString();
						threshActivityDrop = t.alert_activity_drop_pct.toString();
					})
					.catch((e) => console.warn('Failed to load alert thresholds', e));
				lelyLoading = true;
				getLelyConfig()
					.then((c) => {
						lelyConfig = c;
					})
					.catch((e) => console.warn('Failed to load Lely config', e));
				getLelyStatus()
					.then((s) => {
						lelyStatus = s;
					})
					.catch((e) => console.warn('Failed to load Lely status', e))
					.finally(() => (lelyLoading = false));
			}
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
		pendingRoleUserId = userId;
		pendingRoleNew = newRole;
		showRoleConfirm = true;
	}

	async function confirmRoleChange() {
		try {
			await updateUserRole(pendingRoleUserId, pendingRoleNew);
			toasts.success('Роль изменена');
			showRoleConfirm = false;
			load();
		} catch (e) {
			toasts.error(e instanceof Error ? e.message : 'Ошибка');
		}
	}

	async function savePreferences() {
		await preferences.update({
			theme: prefsTheme,
			page_size: prefsPageSize,
			compact_view: prefsCompact,
			language: prefsLang,
		});
		if (prefsTheme !== 'auto') {
			if (prefsTheme === 'dark') {
				document.documentElement.classList.add('dark');
			} else {
				document.documentElement.classList.remove('dark');
			}
			theme.set(prefsTheme as 'light' | 'dark');
		}
		toasts.success('Настройки сохранены');
	}

	async function saveJwtTtl() {
		try {
			jwtSaving = true;
			jwtTtl = await updateJwtTtl({
				jwt_access_ttl_secs: parseInt(jwtAccessTtl) || undefined,
				jwt_refresh_ttl_secs: parseInt(jwtRefreshTtl) || undefined,
			});
			toasts.success('Настройки JWT сохранены');
		} catch (e) {
			toasts.error(e instanceof Error ? e.message : 'Ошибка');
		} finally {
			jwtSaving = false;
		}
	}

	async function saveThresholds() {
		try {
			threshSaving = true;
			thresholds = await updateAlertThresholds({
				alert_min_milk: parseFloat(threshMinMilk) || undefined,
				alert_max_scc: parseFloat(threshMaxScc) || undefined,
				alert_days_before_calving: parseInt(threshDaysCalving) || undefined,
				alert_activity_drop_pct: parseInt(threshActivityDrop) || undefined,
			});
			toasts.success('Пороги уведомлений сохранены');
		} catch (e) {
			toasts.error(e instanceof Error ? e.message : 'Ошибка');
		} finally {
			threshSaving = false;
		}
	}

	function formatUptime(secs: number): string {
		const d = Math.floor(secs / 86400);
		const h = Math.floor((secs % 86400) / 3600);
		const m = Math.floor((secs % 3600) / 60);
		if (d > 0) return `${d}д ${h}ч ${m}м`;
		if (h > 0) return `${h}ч ${m}м`;
		return `${m}м`;
	}

	onMount(load);
</script>

<svelte:head>
	<title>Настройки — Молочная ферма</title>
</svelte:head>

<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100 mb-6">Настройки</h1>

<!-- Tabs -->
<div class="flex gap-1 mb-6 border-b border-slate-200 dark:border-slate-700">
	{#each tabs as tab (tab.key)}
		<button
			onclick={() => (activeTab = tab.key)}
			class="px-4 py-2.5 text-sm font-medium transition-colors cursor-pointer relative {activeTab ===
			tab.key
				? 'text-blue-600 dark:text-blue-400'
				: 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-300'}"
		>
			{tab.label}
			{#if activeTab === tab.key}
				<span
					class="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-600 dark:bg-blue-400 rounded-full"
				></span>
			{/if}
		</button>
	{/each}
</div>

<ErrorAlert message={error} />

<!-- Profile Tab -->
{#if activeTab === 'profile'}
	<section class="max-w-lg space-y-6">
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
		>
			<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-4">Аккаунт</h2>
			<dl class="space-y-3">
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Имя пользователя</dt>
					<dd class="text-sm font-medium text-slate-800 dark:text-slate-100">{$auth.username}</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Роль</dt>
					<dd class="text-sm text-slate-800 dark:text-slate-100">{$auth.role}</dd>
				</div>
			</dl>
		</div>

		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
		>
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
					class="px-3 py-1.5 text-sm bg-slate-100 dark:bg-slate-900 hover:bg-slate-200 dark:hover:bg-slate-700 rounded-lg transition-colors cursor-pointer"
				>
					Сменить
				</button>
			</div>
			<p class="text-sm text-slate-500 dark:text-slate-400">
				Рекомендуется менять пароль каждые 90 дней
			</p>
		</div>
	</section>
{/if}

<!-- Interface Tab -->
{#if activeTab === 'interface'}
	<section class="max-w-lg space-y-6">
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
		>
			<h2
				class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-4 flex items-center gap-2"
			>
				<Settings size={18} /> Внешний вид
			</h2>
			<div class="space-y-5">
				<div>
					<label for="prefs-theme" class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2"
						>Тема</label
					>
					<div class="grid grid-cols-3 gap-2">
						<button
							onclick={() => (prefsTheme = 'light')}
							class="flex items-center justify-center gap-1.5 px-3 py-2 text-sm rounded-lg border transition-colors cursor-pointer {prefsTheme ===
							'light'
								? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400'
								: 'border-slate-300 dark:border-slate-600 hover:bg-slate-50 dark:hover:bg-slate-700'}"
						>
							<Sun size={14} /> Светлая
						</button>
						<button
							onclick={() => (prefsTheme = 'dark')}
							class="flex items-center justify-center gap-1.5 px-3 py-2 text-sm rounded-lg border transition-colors cursor-pointer {prefsTheme ===
							'dark'
								? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400'
								: 'border-slate-300 dark:border-slate-600 hover:bg-slate-50 dark:hover:bg-slate-700'}"
						>
							<Moon size={14} /> Тёмная
						</button>
						<button
							onclick={() => (prefsTheme = 'auto')}
							class="flex items-center justify-center gap-1.5 px-3 py-2 text-sm rounded-lg border transition-colors cursor-pointer {prefsTheme ===
							'auto'
								? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400'
								: 'border-slate-300 dark:border-slate-600 hover:bg-slate-50 dark:hover:bg-slate-700'}"
						>
							<Monitor size={14} /> Авто
						</button>
					</div>
				</div>

				<div>
					<label for="prefs-page-size" class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2"
						>Размер страницы таблицы</label
					>
					<select
						id="prefs-page-size"
						bind:value={prefsPageSize}
						class="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-white dark:bg-slate-900 text-slate-800 dark:text-slate-200 outline-none"
					>
						<option value={20}>20 строк</option>
						<option value={50}>50 строк</option>
						<option value={100}>100 строк</option>
					</select>
				</div>

				<div class="flex items-center justify-between">
					<div>
						<span class="text-sm font-medium text-slate-700 dark:text-slate-300"
							>Компактный вид</span
						>
						<p class="text-xs text-slate-400">Уменьшить отступы в таблицах</p>
					</div>
					<button
						onclick={() => (prefsCompact = !prefsCompact)}
						aria-label={prefsCompact ? 'Выключить компактный вид' : 'Включить компактный вид'}
						class="relative w-10 h-5 rounded-full transition-colors cursor-pointer {prefsCompact
							? 'bg-blue-600'
							: 'bg-slate-300 dark:bg-slate-600'}"
					>
						<span
							class="absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full transition-transform {prefsCompact
								? 'translate-x-5'
								: ''}"
						></span>
					</button>
				</div>

				<div>
					<label for="prefs-language" class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2"
						>Язык</label
					>
					<select
						id="prefs-language"
						bind:value={prefsLang}
						class="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-white dark:bg-slate-900 text-slate-800 dark:text-slate-200 outline-none"
					>
						<option value="ru">Русский</option>
						<option value="en">English</option>
					</select>
				</div>
			</div>

			<div class="flex justify-end mt-6 pt-4 border-t border-slate-200 dark:border-slate-700">
				<button
					onclick={savePreferences}
					class="flex items-center gap-1.5 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-lg transition-colors cursor-pointer"
				>
					<Save size={14} /> Сохранить
				</button>
			</div>
		</div>
	</section>
{/if}

<!-- Users Tab (admin) -->
{#if activeTab === 'users' && isAdmin}
	<section>
		<div class="flex items-center justify-between mb-4">
			<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 flex items-center gap-2">
				<Users size={18} /> Пользователи
			</h2>
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

		<DataTable
			columns={[
				{ key: 'id', label: 'ID' },
				{ key: 'username', label: 'Имя' },
				{ key: 'role', label: 'Роль' },
				{ key: 'created_at', label: 'Создан' },
				{ key: 'actions', label: '', align: 'right' },
			]}
			{loading}
			bind:this={dataTable}
			emptyText="Нет пользователей"
		>
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
							class="text-xs border border-slate-300 dark:border-slate-600 rounded px-1 py-0.5 bg-white dark:bg-slate-900"
						>
							<option value="admin">admin</option>
							<option value="user">user</option>
						</select>
					</td>
					<td class="px-4 py-3 text-slate-500 dark:text-slate-400"
						>{new Date(u.created_at).toLocaleDateString('ru-RU')}</td
					>
					<td class="px-4 py-3 text-right">
						{#if u.username !== $auth.username}
						<button
							onclick={() => {
								deleteUserId = u.id;
								deleteUsername = u.username;
								showDelete = true;
							}}
							class="text-red-500 hover:text-red-700 text-xs cursor-pointer">Удалить</button
						>
						{/if}
					</td>
				</tr>
			{/each}
		</DataTable>
	</section>
{/if}

<!-- System Tab (admin) -->
{#if activeTab === 'system' && isAdmin}
	<section class="space-y-6">
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
		>
			<h2
				class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-4 flex items-center gap-2"
			>
				<BarChart3 size={18} /> Информация о системе
			</h2>
			{#if sysInfo}
				<div class="grid grid-cols-2 md:grid-cols-4 gap-4">
					<div>
						<div class="text-xs text-slate-400">Версия</div>
						<div class="text-sm font-medium text-slate-800 dark:text-slate-100">
							v{sysInfo.version}
						</div>
					</div>
					<div>
						<div class="text-xs text-slate-400">Uptime</div>
						<div class="text-sm font-medium text-slate-800 dark:text-slate-100">
							{formatUptime(sysInfo.uptime_secs)}
						</div>
					</div>
					<div>
						<div class="text-xs text-slate-400">Размер БД</div>
						<div class="text-sm font-medium text-slate-800 dark:text-slate-100">
							{sysInfo.db_size_mb} МБ
						</div>
					</div>
					<div>
						<div class="text-xs text-slate-400">Пользователей</div>
						<div class="text-sm font-medium text-slate-800 dark:text-slate-100">
							{sysInfo.total_users}
						</div>
					</div>
					<div>
						<div class="text-xs text-slate-400">Животных</div>
						<div class="text-sm font-medium text-slate-800 dark:text-slate-100">
							{sysInfo.total_animals}
						</div>
					</div>
					<div>
						<div class="text-xs text-slate-400">Записей надоев</div>
						<div class="text-sm font-medium text-slate-800 dark:text-slate-100">
							{sysInfo.total_milk_records}
						</div>
					</div>
					<div>
						<div class="text-xs text-slate-400">Записей воспр.</div>
						<div class="text-sm font-medium text-slate-800 dark:text-slate-100">
							{sysInfo.total_reproduction_records}
						</div>
					</div>
				</div>
			{:else}
				<div class="animate-pulse space-y-2">
					{#each Array(4) as _, i (i)}
						<div class="h-4 bg-slate-200 dark:bg-slate-700 rounded w-1/3"></div>
					{/each}
				</div>
			{/if}
		</div>

		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
		>
			<h2
				class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-4 flex items-center gap-2"
			>
				<Database size={18} /> Бэкап базы данных
			</h2>
			<p class="text-sm text-slate-500 dark:text-slate-400 mb-4">
				Скачать полную копию базы данных в формате SQL
			</p>
			<a
				href={getBackupUrl()}
				class="inline-flex items-center gap-1.5 px-4 py-2 bg-green-600 hover:bg-green-700 text-white text-sm rounded-lg transition-colors"
			>
				<Download size={14} /> Скачать бэкап
			</a>
		</div>
	</section>
{/if}

<!-- Security Tab (admin) -->
{#if activeTab === 'security' && isAdmin}
	<section class="max-w-lg">
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
		>
			<h2
				class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-4 flex items-center gap-2"
			>
				<Shield size={18} /> Настройки JWT токенов
			</h2>
			{#if jwtTtl}
				<div class="space-y-4">
					<FormField
						id="jwt-access"
						label="Access token TTL (секунд)"
						type="number"
						bind:value={jwtAccessTtl}
					/>
					<FormField
						id="jwt-refresh"
						label="Refresh token TTL (секунд)"
						type="number"
						bind:value={jwtRefreshTtl}
					/>
					<p class="text-xs text-slate-400">
						Текущие: access {jwtTtl.jwt_access_ttl_secs}с, refresh
						{jwtTtl.jwt_refresh_ttl_secs}с
					</p>
					<div class="flex justify-end pt-4 border-t border-slate-200 dark:border-slate-700">
						<button
							onclick={saveJwtTtl}
							disabled={jwtSaving}
							class="flex items-center gap-1.5 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-lg transition-colors cursor-pointer disabled:opacity-50"
						>
							<Save size={14} />
							{jwtSaving ? 'Сохранение...' : 'Сохранить'}
						</button>
					</div>
				</div>
			{/if}
		</div>
	</section>
{/if}

<!-- Alerts Tab (admin) -->
{#if activeTab === 'alerts' && isAdmin}
	<section class="max-w-lg">
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
		>
			<h2
				class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-4 flex items-center gap-2"
			>
				<Bell size={18} /> Пороги уведомлений
			</h2>
			{#if thresholds}
				<div class="space-y-4">
					<FormField
						id="thresh-milk"
						label="Минимальный надой (кг)"
						type="number"
						bind:value={threshMinMilk}
					/>
					<FormField
						id="thresh-scc"
						label="Максимальный СОК"
						type="number"
						bind:value={threshMaxScc}
					/>
					<FormField
						id="thresh-calving"
						label="Дни до отёла (предупреждение)"
						type="number"
						bind:value={threshDaysCalving}
					/>
					<FormField
						id="thresh-activity"
						label="Падение активности (%)"
						type="number"
						bind:value={threshActivityDrop}
					/>
					<div class="flex justify-end pt-4 border-t border-slate-200 dark:border-slate-700">
						<button
							onclick={saveThresholds}
							disabled={threshSaving}
							class="flex items-center gap-1.5 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-lg transition-colors cursor-pointer disabled:opacity-50"
						>
							<Save size={14} />
							{threshSaving ? 'Сохранение...' : 'Сохранить'}
						</button>
					</div>
				</div>
			{/if}
		</div>
	</section>
{/if}

<!-- Lely Integration Tab (admin) -->
{#if activeTab === 'lely' && isAdmin}
	<section class="space-y-6">
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
		>
			<h2
				class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-4 flex items-center gap-2"
			>
				<Plug size={18} /> Подключение Lely
			</h2>
			{#if lelyConfig}
				<div class="space-y-3 text-sm">
					<div class="flex justify-between">
						<span class="text-slate-500 dark:text-slate-400">Статус</span>
						<span
							class="px-2 py-0.5 rounded text-xs font-medium {lelyConfig.enabled
								? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
								: 'bg-slate-100 text-slate-600 dark:bg-slate-700 dark:text-slate-400'}"
						>
							{lelyConfig.enabled ? 'Включена' : 'Отключена'}
						</span>
					</div>
					<div class="flex justify-between">
						<span class="text-slate-500 dark:text-slate-400">URL сервера</span>
						<span class="text-slate-800 dark:text-slate-200">{lelyConfig.base_url || '—'}</span>
					</div>
					<div class="flex justify-between">
						<span class="text-slate-500 dark:text-slate-400">Пользователь</span>
						<span class="text-slate-800 dark:text-slate-200">{lelyConfig.username || '—'}</span>
					</div>
					<div class="flex justify-between">
						<span class="text-slate-500 dark:text-slate-400">FarmKey</span>
						<span class="text-slate-800 dark:text-slate-200"
							>{lelyConfig.farm_key_set ? '••••••' : 'Не задан'}</span
						>
					</div>
					<div class="flex justify-between">
						<span class="text-slate-500 dark:text-slate-400">Интервал синхр.</span>
						<span class="text-slate-800 dark:text-slate-200"
							>{lelyConfig.sync_interval_secs < 60
								? `${lelyConfig.sync_interval_secs}с`
								: `${Math.floor(lelyConfig.sync_interval_secs / 60)}м`}</span
						>
					</div>
					<p class="text-xs text-slate-400 pt-2">
						Настройки задаются через переменные окружения (LELY_*)
					</p>
				</div>
			{:else}
				<div class="animate-pulse space-y-2">
					{#each Array(4) as _, i (i)}
						<div class="h-4 bg-slate-200 dark:bg-slate-700 rounded w-2/3"></div>
					{/each}
				</div>
			{/if}

			<div class="flex justify-end mt-4 pt-4 border-t border-slate-200 dark:border-slate-700">
				<button
					onclick={async () => {
						try {
							lelySyncing = true;
							await triggerLelySync();
							toasts.success('Синхронизация запущена');
						} catch (e) {
							toasts.error(e instanceof Error ? e.message : 'Ошибка');
						} finally {
							lelySyncing = false;
						}
					}}
					disabled={lelySyncing || !lelyConfig?.enabled}
					class="flex items-center gap-1.5 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-lg transition-colors cursor-pointer disabled:opacity-50"
				>
					<RefreshCw size={14} />
					{lelySyncing ? 'Запуск...' : 'Синхронизировать'}
				</button>
			</div>
		</div>

		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
		>
			<h2
				class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-4 flex items-center gap-2"
			>
				<RefreshCw size={18} /> Статус синхронизации
			</h2>
			{#if lelyLoading}
				<div class="animate-pulse space-y-2">
					{#each Array(6) as _, i (i)}
						<div class="h-4 bg-slate-200 dark:bg-slate-700 rounded w-full"></div>
					{/each}
				</div>
			{:else if lelyStatus.length > 0}
				<div class="overflow-x-auto">
					<table class="w-full text-sm">
						<thead>
							<tr class="border-b border-slate-200 dark:border-slate-700">
								<th class="text-left py-2 px-2 text-slate-500 dark:text-slate-400 font-medium"
									>Тип</th
								>
								<th class="text-left py-2 px-2 text-slate-500 dark:text-slate-400 font-medium"
									>Статус</th
								>
								<th class="text-right py-2 px-2 text-slate-500 dark:text-slate-400 font-medium"
									>Записей</th
								>
								<th class="text-left py-2 px-2 text-slate-500 dark:text-slate-400 font-medium"
									>Последняя синхр.</th
								>
								<th class="text-left py-2 px-2 text-slate-500 dark:text-slate-400 font-medium"
									>Ошибка</th
								>
							</tr>
						</thead>
						<tbody>
							{#each lelyStatus as s (s.entity_type)}
								<tr
									class="border-b border-slate-100 dark:border-slate-700/50 hover:bg-slate-50 dark:hover:bg-slate-800/50"
								>
									<td class="py-2 px-2 font-mono text-xs">{s.entity_type}</td>
									<td class="py-2 px-2">
										<span
											class="px-1.5 py-0.5 rounded text-xs font-medium {s.status ===
											'success'
												? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
												: s.status === 'error'
													? 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'
													: 'bg-slate-100 text-slate-600 dark:bg-slate-700 dark:text-slate-400'}"
										>
											{s.status === 'success'
												? 'ОК'
												: s.status === 'error'
													? 'Ошибка'
													: s.status === 'running'
														? 'Выполняется'
														: 'Ожидание'}
										</span>
									</td>
									<td class="py-2 px-2 text-right text-slate-600 dark:text-slate-300"
										>{s.records_synced}</td
									>
									<td class="py-2 px-2 text-slate-500 dark:text-slate-400 text-xs"
										>{s.last_synced_at
											? new Date(s.last_synced_at).toLocaleString('ru-RU')
											: '—'}</td
									>
									<td class="py-2 px-2 text-red-500 text-xs max-w-[200px] truncate"
										>{s.error_message || ''}</td
									>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{:else}
				<p class="text-sm text-slate-400">Нет данных о синхронизации</p>
			{/if}
		</div>
	</section>
{/if}

<!-- Modals -->
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
			>
				Отмена
			</button>
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
			>
				Отмена
			</button>
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

<ConfirmDialog
	open={showRoleConfirm}
	title="Изменить роль"
	message="Подтвердите изменение роли пользователя."
	confirmText="Изменить"
	destructive={false}
	onconfirm={confirmRoleChange}
	oncancel={() => (showRoleConfirm = false)}
/>
