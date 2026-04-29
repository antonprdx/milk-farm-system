/* eslint-disable @typescript-eslint/no-explicit-any */
export function themeColors(isDark: boolean) {
	return {
		gridColor: isDark ? 'rgba(148,163,184,0.15)' : 'rgba(203,213,225,0.5)',
		textColor: isDark ? '#94a3b8' : '#64748b',
	};
}

export function defaultTooltip(
	isDark: boolean,
	callbacks?: Record<string, (ctx: unknown) => string>,
) {
	return {
		backgroundColor: isDark ? '#1e293b' : '#fff',
		titleColor: isDark ? '#e2e8f0' : '#1e293b',
		bodyColor: isDark ? '#94a3b8' : '#475569',
		borderColor: isDark ? '#334155' : '#e2e8f0',
		borderWidth: 1,
		padding: 10,
		cornerRadius: 8,
		...(callbacks ? { callbacks } : {}),
	};
}

export function defaultScales(
	isDark: boolean,
	yCallback?: (v: string | number) => string | number | undefined,
) {
	const { gridColor, textColor } = themeColors(isDark);
	return {
		x: {
			grid: { display: false },
			ticks: { color: textColor, maxRotation: 45, font: { size: 11 } },
		},
		y: {
			beginAtZero: true as const,
			grid: { color: gridColor },
			ticks: {
				color: textColor,
				font: { size: 11 },
				...(yCallback ? { callback: yCallback as any } : {}),
			},
		},
	};
}

export function dsColors(isDark: boolean, hue: 'blue' | 'red' | 'green' | 'purple' | 'cyan' | 'orange' | 'pink' | 'yellow' = 'blue') {
	const map: Record<string, { border: string; bg: string; point: string }> = {
		blue: {
			border: isDark ? 'rgba(96,165,250,0.9)' : 'rgba(37,99,235,0.9)',
			bg: isDark ? 'rgba(59,130,246,0.15)' : 'rgba(59,130,246,0.1)',
			point: isDark ? 'rgba(96,165,250,0.9)' : 'rgba(37,99,235,0.9)',
		},
		red: {
			border: isDark ? 'rgba(248,113,113,0.9)' : 'rgba(220,38,38,0.9)',
			bg: isDark ? 'rgba(248,113,113,0.15)' : 'rgba(220,38,38,0.1)',
			point: isDark ? 'rgba(248,113,113,0.9)' : 'rgba(220,38,38,0.9)',
		},
		green: {
			border: isDark ? 'rgba(52,211,153,1)' : 'rgba(5,150,105,1)',
			bg: isDark ? 'rgba(52,211,153,0.08)' : 'rgba(5,150,105,0.08)',
			point: isDark ? 'rgba(52,211,153,1)' : 'rgba(5,150,105,1)',
		},
		purple: {
			border: isDark ? 'rgba(168,139,250,0.9)' : 'rgba(124,58,237,0.9)',
			bg: isDark ? 'rgba(139,92,246,0.15)' : 'rgba(139,92,246,0.1)',
			point: isDark ? 'rgba(168,139,250,0.9)' : 'rgba(124,58,237,0.9)',
		},
		cyan: {
			border: isDark ? 'rgba(34,211,238,0.9)' : 'rgba(8,145,178,0.9)',
			bg: isDark ? 'rgba(34,211,238,0.15)' : 'rgba(34,211,238,0.1)',
			point: isDark ? 'rgba(34,211,238,0.9)' : 'rgba(8,145,178,0.9)',
		},
		orange: {
			border: isDark ? 'rgba(251,146,60,0.9)' : 'rgba(234,88,12,0.9)',
			bg: isDark ? 'rgba(251,146,60,0.15)' : 'rgba(251,146,60,0.1)',
			point: isDark ? 'rgba(251,146,60,0.9)' : 'rgba(234,88,12,0.9)',
		},
		pink: {
			border: isDark ? 'rgba(244,114,182,0.9)' : 'rgba(219,39,119,0.9)',
			bg: isDark ? 'rgba(244,114,182,0.15)' : 'rgba(244,114,182,0.1)',
			point: isDark ? 'rgba(244,114,182,0.9)' : 'rgba(219,39,119,0.9)',
		},
		yellow: {
			border: isDark ? 'rgba(250,204,21,0.9)' : 'rgba(202,138,4,0.9)',
			bg: isDark ? 'rgba(250,204,21,0.15)' : 'rgba(250,204,21,0.1)',
			point: isDark ? 'rgba(250,204,21,0.9)' : 'rgba(202,138,4,0.9)',
		},
	};
	return map[hue];
}
