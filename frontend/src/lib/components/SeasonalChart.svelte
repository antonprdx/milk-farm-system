<script module lang="ts">
	import type { SeasonalResponse } from '$lib/api/analytics';
</script>

<script lang="ts">
	import {
		Chart,
		BarController,
		CategoryScale,
		LinearScale,
		BarElement,
		Tooltip,
		Legend,
	} from 'chart.js';
	import { theme } from '$lib/stores/theme';
	import { themeColors, defaultTooltip } from '$lib/utils/chartHelpers';
	import { debounce } from '$lib/utils/debounce';

	Chart.register(
		BarController,
		CategoryScale,
		LinearScale,
		BarElement,
		Tooltip,
		Legend,
	);

	interface Props {
		data: SeasonalResponse;
	}

	let { data }: Props = $props();

	let canvas: HTMLCanvasElement | undefined = $state();
	let chart: Chart | null = null;

	function buildChart() {
		if (!canvas) return;

		if (chart) {
			chart.destroy();
			chart = null;
		}

		const isDark = $theme === 'dark';
		const { gridColor, textColor } = themeColors(isDark);

		const labels = data.monthly_indices.map((m) => m.month_name.slice(0, 3));
		const values = data.monthly_indices.map((m) => m.seasonal_index ?? 1);
		const bgColors = values.map((v) =>
			v >= 1
				? isDark
					? 'rgba(52,211,153,0.6)'
					: 'rgba(5,150,105,0.6)'
				: isDark
					? 'rgba(248,113,113,0.6)'
					: 'rgba(220,38,38,0.5)',
		);
		const borderColors = values.map((v) =>
			v >= 1
				? isDark
					? 'rgba(52,211,153,0.9)'
					: 'rgba(5,150,105,0.9)'
				: isDark
					? 'rgba(248,113,113,0.9)'
					: 'rgba(220,38,38,0.9)',
		);

		chart = new Chart(canvas, {
			type: 'bar',
			data: {
				labels,
				datasets: [
					{
						label: 'Сезонный индекс',
						data: values as number[],
						backgroundColor: bgColors,
						borderColor: borderColors,
						borderWidth: 1,
						borderRadius: 4,
						maxBarThickness: 50,
					},
				],
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				plugins: {
					legend: { display: false },
					tooltip: defaultTooltip(isDark, {
						label: (ctx: unknown) => {
							const c = ctx as { parsed: { y: number } };
							return `Индекс: ${c.parsed.y.toFixed(2)}`;
						},
					}),
				},
				scales: {
					x: {
						grid: { display: false },
						ticks: { color: textColor, font: { size: 11 } },
					},
					y: {
						beginAtZero: false,
						suggestedMin: 0.8,
						suggestedMax: 1.2,
						grid: { color: gridColor },
						ticks: { color: textColor, font: { size: 11 } },
					},
				},
			},
		});
	}

	let debouncedBuild = debounce(() => buildChart(), 50);

	$effect(() => {
		data;
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

<div class="relative h-full">
	<canvas bind:this={canvas}></canvas>
</div>
