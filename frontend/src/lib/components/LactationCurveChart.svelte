<script module lang="ts">
	import type { LactationCurveResponse } from '$lib/api/analytics';
</script>

<script lang="ts">
	import {
		Chart,
		LineController,
		CategoryScale,
		LinearScale,
		PointElement,
		LineElement,
		Filler,
		Tooltip,
		Legend,
	} from 'chart.js';
	import { theme } from '$lib/stores/theme';
	import { themeColors, defaultTooltip, dsColors } from '$lib/utils/chartHelpers';
	import { debounce } from '$lib/utils/debounce';

	Chart.register(
		LineController,
		CategoryScale,
		LinearScale,
		PointElement,
		LineElement,
		Filler,
		Tooltip,
		Legend,
	);

	interface Props {
		data: LactationCurveResponse;
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
		const blue = dsColors(isDark, 'blue');
		const red = dsColors(isDark, 'red');
		const green = dsColors(isDark, 'green');

		const allDims = new Set<number>();
		for (const p of data.actual_points) allDims.add(p.dim);
		for (const p of data.fitted_curve) allDims.add(p.dim);
		for (const p of data.forecast) allDims.add(p.dim);
		const labels = [...allDims].sort((a, b) => a - b);

		const actualMap = new Map(data.actual_points.map((p: { dim: number; milk: number | null }) => [p.dim, p.milk]));
		const fittedMap = new Map(data.fitted_curve.map((p: { dim: number; milk: number | null }) => [p.dim, p.milk]));
		const forecastMap = new Map(data.forecast.map((p: { dim: number; milk: number | null }) => [p.dim, p.milk]));

		const actualData: (number | null)[] = labels.map((d) => actualMap.get(d) ?? null);
		const fittedData: (number | null)[] = labels.map((d) => fittedMap.get(d) ?? null);
		const forecastData: (number | null)[] = labels.map((d) => forecastMap.get(d) ?? null);

		chart = new Chart(canvas, {
			type: 'line',
			data: {
				labels: labels.map(String),
				datasets: [
					{
						label: 'Факт',
						data: actualData,
						borderColor: blue.border,
						backgroundColor: blue.bg,
						pointRadius: 2,
						pointBackgroundColor: blue.point,
						borderWidth: 2,
						tension: 0.3,
						spanGaps: true,
						fill: false,
					},
					{
						label: 'Модель Wood',
						data: fittedData,
						borderColor: green.border,
						backgroundColor: green.bg,
						pointRadius: 0,
						borderWidth: 2,
						tension: 0.3,
						spanGaps: true,
						fill: false,
					},
					{
						label: 'Прогноз',
						data: forecastData,
						borderColor: red.border,
						backgroundColor: red.bg,
						pointRadius: 0,
						borderWidth: 2,
						borderDash: [6, 3],
						tension: 0.3,
						spanGaps: true,
						fill: false,
					},
				],
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				plugins: {
					legend: {
						labels: { color: textColor, font: { size: 11 } },
					},
					tooltip: defaultTooltip(isDark, {
						label: (ctx: unknown) => {
							const c = ctx as { dataset: { label: string }; parsed: { y: number | null } };
							return `${c.dataset.label}: ${c.parsed.y?.toFixed(1) ?? '—'} л`;
						},
					}),
				},
				scales: {
					x: {
						title: { display: true, text: 'DIM (дни в лактации)', color: textColor },
						grid: { display: false },
						ticks: {
							color: textColor,
							font: { size: 10 },
							maxTicksLimit: 20,
						},
					},
					y: {
						title: { display: true, text: 'Надой, л', color: textColor },
						beginAtZero: true,
						grid: { color: gridColor },
						ticks: { color: textColor, font: { size: 10 } },
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
