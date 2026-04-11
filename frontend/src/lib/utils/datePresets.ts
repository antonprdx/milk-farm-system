export function todayStr(): string {
	return new Date().toISOString().slice(0, 10);
}

export function daysAgoStr(days: number): string {
	const d = new Date();
	d.setDate(d.getDate() - days + 1);
	return d.toISOString().slice(0, 10);
}

export function monthStartStr(): string {
	const d = new Date();
	d.setDate(1);
	return d.toISOString().slice(0, 10);
}
