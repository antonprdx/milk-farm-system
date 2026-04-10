<script lang="ts">
	import type { Snippet } from 'svelte';

	type Column = {
		key: string;
		label: string;
		align?: 'left' | 'right' | 'center';
	};

	let {
		columns = [],
		colspan,
		loading = false,
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		emptyText = 'Нет данных',
		children,
	}: {
		columns: Column[];
		colspan?: number;
		loading?: boolean;
		emptyText?: string;
		children: Snippet;
	} = $props();

	let cols = $derived(colspan ?? columns.length);
</script>

<div
	class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 overflow-hidden"
>
	<div class="overflow-x-auto">
		<table class="w-full text-sm">
			<thead
				class="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-200 dark:border-slate-700"
			>
				<tr>
					{#each columns as col (col.key)}
						<th
							class="{col.align === 'right'
								? 'text-right'
								: col.align === 'center'
									? 'text-center'
									: 'text-left'} px-4 py-3 text-slate-600 dark:text-slate-400 font-medium"
						>{col.label}</th>
					{/each}
				</tr>
			</thead>
			<tbody class="[&>tr:nth-child(even)]:bg-slate-50/50 dark:[&>tr:nth-child(even)]:bg-slate-800/30">
				{#if loading}
					{#each Array(5) as _, i (i)}
						<tr class="border-b border-slate-100 dark:border-slate-700 {i % 2 === 1 ? 'bg-slate-50/50 dark:bg-slate-800/30' : ''}">
							{#each Array(cols) as _, j (j)}
								<td class="px-4 py-3"
									><div class="h-4 bg-slate-100 dark:bg-slate-900 rounded animate-pulse"></div></td
								>
							{/each}
						</tr>
					{/each}
				{:else}
					{@render children()}
				{/if}
			</tbody>
		</table>
	</div>
</div>
