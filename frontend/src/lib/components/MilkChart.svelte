<script module lang="ts">
	/* eslint-disable svelte/prefer-svelte-reactivity */
	import type { MilkDayProduction } from '$lib/api/milk';
</script>

<script lang="ts">
	import { Chart, BarController, CategoryScale, LinearScale, BarElement, Tooltip } from 'chart.js';
	import { theme } from '$lib/stores/theme';
	import { defaultTooltip, defaultScales } from '$lib/utils/chartHelpers';
	import { debounce } from '$lib/utils/debounce';

	Chart.register(BarController, CategoryScale, LinearScale, BarElement, Tooltip);

	type Granularity = 'day' | 'week';

	interface Props {
		productions: MilkDayProduction[];
	}

	let { productions }: Props = $props();

	let canvas: HTMLCanvasElement | undefined = $state();
	let chart: Chart | null = null;
	let granularity = $state<Granularity>('day');

	function aggregateByDay(data: MilkDayProduction[]) {
		const map = new Map<string, number>();
		for (const p of data) {
			const key = p.date;
			map.set(key, (map.get(key) ?? 0) + (p.milk_amount ?? 0));
		}
		const entries = [...map.entries()].sort(([a], [b]) => a.localeCompare(b));
		return {
			labels: entries.map(([d]) => d),
			values: entries.map(([, v]) => Math.round(v * 100) / 100),
		};
	}

	function aggregateByWeek(data: MilkDayProduction[]) {
		const map = new Map<string, { sum: number; count: number }>();
		for (const p of data) {
			const d = new Date(p.date + 'T00:00:00');
			const dayOfWeek = d.getDay();
			const mondayOffset = dayOfWeek === 0 ? -6 : 1 - dayOfWeek;
			const monday = new Date(d);
			monday.setDate(d.getDate() + mondayOffset);
			const key = monday.toISOString().slice(0, 10);
			const entry = map.get(key) ?? { sum: 0, count: 0 };
			entry.sum += p.milk_amount ?? 0;
			entry.count += 1;
			map.set(key, entry);
		}
		const entries = [...map.entries()].sort(([a], [b]) => a.localeCompare(b));
		return {
			labels: entries.map(([start]) => {
				const s = new Date(start + 'T00:00:00');
				const end = new Date(s);
				end.setDate(s.getDate() + 6);
				return `${s.toLocaleDateString('ru-RU', { day: 'numeric', month: 'short' })} — ${end.toLocaleDateString('ru-RU', { day: 'numeric', month: 'short' })}`;
			}),
			values: entries.map(([, v]) => Math.round(v.sum * 100) / 100),
		};
	}

	function buildChart() {
		if (!canvas) return;

		if (chart) {
			chart.destroy();
			chart = null;
		}

		const isDark = $theme === 'dark';

		const { labels, values } =
			granularity === 'day' ? aggregateByDay(productions) : aggregateByWeek(productions);

		if (labels.length === 0) return;

		chart = new Chart(canvas, {
			type: 'bar',
			data: {
				labels,
				datasets: [
					{
						label: granularity === 'day' ? 'Надой, л' : 'Надой за неделю, л',
						data: values,
						backgroundColor: isDark ? 'rgba(59,130,246,0.6)' : 'rgba(59,130,246,0.7)',
						borderColor: isDark ? 'rgba(96,165,250,0.9)' : 'rgba(37,99,235,0.9)',
						borderWidth: 1,
						borderRadius: 4,
						maxBarThickness: 60,
					},
				],
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				plugins: {
					legend: { display: false },
					tooltip: defaultTooltip(isDark, {
						// eslint-disable-next-line @typescript-eslint/no-explicit-any
						label: (ctx: any) => `${ctx.parsed.y?.toFixed(1) ?? '0'} л`,
					}),
				},
				scales: defaultScales(isDark, (v) => `${v} л`),
			},
		});
	}

	let debouncedBuild = debounce(() => buildChart(), 50);

	$effect(() => {
		productions;
		granularity;
		$theme;
		debouncedBuild();
	});

	$effect(() => {
		return () => {
			if (chart) {
				chart.destroy();
				chart = null;
			}
		};
	});
</script>

<div
	class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4 mb-6"
>
	<div class="flex items-center justify-between mb-3">
		<h2 class="text-sm font-semibold text-slate-700 dark:text-slate-300">График удоев</h2>
		<div class="flex gap-1">
			<button
				onclick={() => (granularity = 'day')}
				class="px-3 py-1 text-xs font-medium rounded-md transition-colors cursor-pointer {granularity ===
				'day'
					? 'bg-blue-600 text-white'
					: 'bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-slate-600'}"
				>По дням</button
			>
			<button
				onclick={() => (granularity = 'week')}
				class="px-3 py-1 text-xs font-medium rounded-md transition-colors cursor-pointer {granularity ===
				'week'
					? 'bg-blue-600 text-white'
					: 'bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-slate-600'}"
				>По неделям</button
			>
		</div>
	</div>
	<div class="relative h-72">
		{#if productions.length === 0}
			<div
				class="flex items-center justify-center h-full text-slate-400 dark:text-slate-500 text-sm"
			>
				Нет данных для графика
			</div>
		{:else}
			<canvas bind:this={canvas}></canvas>
		{/if}
	</div>
</div>
