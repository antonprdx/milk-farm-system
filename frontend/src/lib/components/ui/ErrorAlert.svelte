<script lang="ts">
	let {
		message = '',
	}: {
		message?: string;
	} = $props();
</script>

{#if message}
	<div
		class="bg-red-50 dark:bg-red-900/50 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-400 px-4 py-3 rounded-lg text-sm mb-4"
	>
		{message}
	</div>
{/if}
