export function fmtNum(v: number | null | undefined, decimals = 1): string {
	return v != null ? v.toFixed(decimals) : '—';
}

export function formatDate(d: string): string {
	return d;
}

export function formatDatetime(dt: string): string {
	return new Date(dt).toLocaleString('ru-RU');
}
