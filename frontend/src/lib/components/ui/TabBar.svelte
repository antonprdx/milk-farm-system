<script lang="ts">
	type Tab = { key: string; label: string };

	let {
		tabs = [],
		active = $bindable(''),
		onchange,
	}: {
		tabs: Tab[];
		active?: string;
		onchange?: (key: string) => void;
	} = $props();

	function select(key: string) {
		active = key;
		onchange?.(key);
	}
</script>

<div class="flex gap-1 mb-4 border-b border-slate-200 dark:border-slate-700" role="tablist">
	{#each tabs as tab (tab.key)}
		<button
			onclick={() => select(tab.key)}
			role="tab"
			aria-selected={active === tab.key}
			class="px-4 py-2.5 text-sm font-medium transition-colors cursor-pointer relative {active ===
			tab.key
				? 'text-blue-600 dark:text-blue-400'
				: 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-300'}"
		>
			{tab.label}
			{#if active === tab.key}
				<span
					class="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-600 dark:bg-blue-400 rounded-full"
				></span>
			{/if}
		</button>
	{/each}
</div>
