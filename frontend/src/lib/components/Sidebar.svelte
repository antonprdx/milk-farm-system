<script lang="ts">
	import { auth } from '$lib/stores/auth';
	import { theme } from '$lib/stores/theme';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { logout as apiLogout } from '$lib/api/auth';
	import {
		LayoutDashboard,
		Beef,
		Milk,
		Heart,
		Wheat,
		Activity,
		TreePine,
		Container,
		MapPin,
		Users,
		BarChart3,
		TrendingUp,
		Settings,
		Sun,
		Moon,
		LogOut,
		PanelLeftClose,
		PanelLeftOpen,
		X,
		Stethoscope,
	} from 'lucide-svelte';

	let {
		collapsed = $bindable(false),
		isAdmin = false,
		onclose,
	}: {
		collapsed?: boolean;
		isAdmin?: boolean;
		onclose?: () => void;
	} = $props();

	interface NavItem {
		href: string;
		label: string;
		icon: typeof LayoutDashboard;
		adminOnly?: boolean;
	}

	const navItems: NavItem[] = [
		{ href: '/', label: 'Дашборд', icon: LayoutDashboard },
		{ href: '/animals', label: 'Животные', icon: Beef },
		{ href: '/milk', label: 'Удои', icon: Milk },
		{ href: '/reproduction', label: 'Воспроизводство', icon: Heart },
		{ href: '/feed', label: 'Кормление', icon: Wheat },
		{ href: '/vet', label: 'Вет. журнал', icon: Stethoscope },
		{ href: '/fitness', label: 'Фитнес', icon: Activity },
		{ href: '/grazing', label: 'Пастьба', icon: TreePine },
		{ href: '/bulk-tank', label: 'Танк-охладитель', icon: Container },
		{ href: '/contacts', label: 'Контакты', icon: Users },
		{ href: '/locations', label: 'Локации', icon: MapPin },
		{ href: '/reports', label: 'Отчёты', icon: BarChart3 },
		{ href: '/analytics', label: 'Аналитика', icon: TrendingUp },
		{ href: '/settings', label: 'Настройки', icon: Settings, adminOnly: true },
	];

	function isActive(href: string, pathname: string): boolean {
		if (href === '/') return pathname === '/';
		return pathname.startsWith(href);
	}

	async function handleLogout() {
		try {
			await apiLogout();
		} catch {
			// ignore
		}
		auth.logout();
		goto('/auth/login');
	}

	function handleNavClick() {
		if (onclose) onclose();
	}
</script>

<nav
	class="fixed left-0 top-0 h-full bg-slate-800 dark:bg-slate-950 text-white transition-all duration-300 flex flex-col z-50 {collapsed
		? 'w-16'
		: 'w-56'}"
>
	<div
		class="flex items-center justify-between p-4 border-b border-slate-700 dark:border-slate-800"
	>
		{#if !collapsed}
			<span class="text-lg font-bold whitespace-nowrap">Молочная ферма</span>
		{/if}
		<button
			onclick={onclose ? onclose : () => (collapsed = !collapsed)}
			class="p-1.5 rounded hover:bg-slate-700 dark:hover:bg-slate-800 text-slate-400 hover:text-white cursor-pointer"
		>
			{#if onclose}
				<X size={18} />
			{:else if collapsed}
				<PanelLeftOpen size={18} />
			{:else}
				<PanelLeftClose size={18} />
			{/if}
		</button>
	</div>

	<div class="flex-1 py-2 overflow-y-auto">
		{#each navItems as item (item.href)}
			{#if !item.adminOnly || isAdmin}
				{@const Icon = item.icon}
				<a
					href={item.href}
					onclick={handleNavClick}
					class="flex items-center gap-3 px-4 py-3 transition-colors {isActive(
						item.href,
						$page.url.pathname,
					)
						? 'bg-slate-700 dark:bg-slate-800 text-white'
						: 'text-slate-300 hover:bg-slate-700 dark:hover:bg-slate-800 hover:text-white'}"
					aria-current={isActive(item.href, $page.url.pathname) ? 'page' : undefined}
				>
					<Icon size={20} class="flex-shrink-0" />
					{#if !collapsed}
						<span class="whitespace-nowrap">{item.label}</span>
					{/if}
				</a>
			{/if}
		{/each}
	</div>

	<div class="border-t border-slate-700 dark:border-slate-800 p-4">
		{#if !collapsed}
			<div class="text-sm text-slate-400 mb-2 truncate">{$auth.username ?? ''}</div>
		{/if}
		<div class="flex gap-2">
			<button
				onclick={() => theme.toggle()}
				class="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 text-sm bg-slate-700 dark:bg-slate-800 hover:bg-slate-600 dark:hover:bg-slate-700 rounded transition-colors cursor-pointer"
				title="Сменить тему"
			>
				{#if $theme === 'dark'}
					<Sun size={14} />
				{:else}
					<Moon size={14} />
				{/if}
				{#if !collapsed}<span>{$theme === 'dark' ? 'Светлая' : 'Тёмная'}</span>{/if}
			</button>
			<button
				onclick={handleLogout}
				class="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 text-sm bg-red-600 hover:bg-red-700 rounded transition-colors cursor-pointer"
			>
				<LogOut size={14} />
				{#if !collapsed}<span>Выйти</span>{/if}
			</button>
		</div>
	</div>
</nav>
