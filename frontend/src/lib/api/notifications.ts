import { api, post, del } from './client';

export interface NotificationChannel {
	id: number;
	user_id: number;
	channel_type: 'browser' | 'telegram';
	channel_token: string;
	enabled: boolean;
}

export interface NotificationRule {
	id: number;
	user_id: number;
	event_type: string;
	channel_id: number | null;
	enabled: boolean;
}

export function listChannels() {
	return api<{ data: NotificationChannel[] }>('/notifications/channels');
}

export function createChannel(channel_type: string, channel_token: string) {
	return post<{ data: NotificationChannel }>('/notifications/channels', { channel_type, channel_token });
}

export function deleteChannel(id: number) {
	return del<{ message: string }>(`/notifications/channels/${id}`);
}

export function listRules() {
	return api<{ data: NotificationRule[] }>('/notifications/rules');
}

export function createRule(event_type: string, channel_id?: number) {
	return post<{ message: string }>('/notifications/rules', { event_type, channel_id });
}

export function deleteRule(id: number) {
	return del<{ message: string }>(`/notifications/rules/${id}`);
}

export const ALERT_EVENTS = [
	{ value: 'all', label: 'Все события' },
	{ value: 'milk_drop', label: 'Снижение надоя' },
	{ value: 'high_scc', label: 'Высокий SCC' },
	{ value: 'activity_drop', label: 'Снижение активности' },
	{ value: 'low_feed', label: 'Снижение корма' },
	{ value: 'no_milking', label: 'Нет доения' },
	{ value: 'expected_calving', label: 'Ожидается отёл' },
	{ value: 'mastitis_risk', label: 'Риск мастита' },
	{ value: 'ketosis_risk', label: 'Риск кетоза' },
];
