<script lang="ts">
	import '../app.css';
	import { auth } from '$lib/stores/auth';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import Toaster from '$lib/components/ui/Toaster.svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { browser } from '$app/environment';
	import { theme } from '$lib/stores/theme';
	import { Menu } from 'lucide-svelte';

	let { children } = $props();

	let isAuthenticated = $derived(browser ? $auth.authenticated : false);
	let pathname = $derived($page.url.pathname);
	let isAdmin = $derived(browser ? $auth.role === 'admin' : false);
	let mustChangePassword = $derived(browser ? $auth.mustChangePassword : false);
	let sidebarCollapsed = $state(false);
	let mobileMenuOpen = $state(false);
	let mobileMenuClosing = $state(false);

	function closeMobileMenu() {
		mobileMenuClosing = true;
		setTimeout(() => {
			mobileMenuOpen = false;
			mobileMenuClosing = false;
		}, 200);
	}

	$effect(() => {
		if (browser && !isAuthenticated && !pathname.startsWith('/auth')) {
			goto('/auth/login');
		}
	});

	$effect(() => {
		if (browser) {
			theme.init();
		}
	});
</script>

<svelte:head>
	<title>Молочная ферма</title>
</svelte:head>

{#if browser}
	<!-- eslint-disable svelte/no-at-html-tags -->
	{@html '<script>(() => { const t = localStorage.getItem("theme"); if (t === "dark") document.documentElement.classList.add("dark"); })()</script>'}
{/if}

{#if isAuthenticated}
	<div class="flex h-screen bg-slate-100 dark:bg-slate-900">
		{#if mobileMenuOpen}
			<div
				class="fixed inset-0 bg-black/50 z-40 md:hidden {mobileMenuClosing
					? 'mobile-overlay-exit'
					: 'mobile-overlay-enter'}"
				onclick={closeMobileMenu}
				onkeydown={(e) => e.key === 'Escape' && closeMobileMenu()}
				role="presentation"
			></div>
		{/if}

		<div class="hidden md:block">
			<Sidebar bind:collapsed={sidebarCollapsed} {isAdmin} />
		</div>

		{#if mobileMenuOpen}
			<div
				class="fixed inset-y-0 left-0 z-50 md:hidden {mobileMenuClosing
					? 'mobile-sidebar-exit'
					: 'mobile-sidebar-enter'}"
			>
				<Sidebar collapsed={false} {isAdmin} onclose={closeMobileMenu} />
			</div>
		{/if}

		<div class="flex-1 flex flex-col min-w-0">
			<header
				class="md:hidden flex items-center gap-3 px-4 py-3 bg-white dark:bg-slate-800 border-b border-slate-200 dark:border-slate-700"
			>
				<button
					onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
					class="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-700 cursor-pointer"
					aria-label="Меню"
				>
					<Menu size={24} class="text-slate-700 dark:text-slate-300" />
				</button>
				<span class="text-lg font-bold text-slate-800 dark:text-white">Молочная ферма</span>
			</header>

			<main
				class="flex-1 overflow-auto p-4 md:p-6 md:transition-all md:duration-300 main-with-sidebar"
				style="--sidebar-w: {sidebarCollapsed ? '4rem' : '14rem'}"
			>
				{#if mustChangePassword && !pathname.startsWith('/settings')}
					<div
						class="mb-4 p-3 bg-amber-50 dark:bg-amber-900/30 border border-amber-200 dark:border-amber-700 rounded-lg flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2"
					>
						<span class="text-sm text-amber-800 dark:text-amber-200"
							>Необходимо сменить пароль для безопасной работы.</span
						>
						<button
							onclick={() => goto('/settings')}
							class="px-3 py-1 text-sm bg-amber-600 hover:bg-amber-700 text-white rounded-lg transition-colors cursor-pointer"
							>Сменить пароль</button
						>
					</div>
				{/if}
				{@render children()}
			</main>
		</div>
	</div>
{:else}
	{@render children()}
{/if}

<Toaster />
