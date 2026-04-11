import { api, post } from './client';

export interface LelySyncStatus {
	entity_type: string;
	last_synced_at: string | null;
	status: string;
	records_synced: number;
	error_message: string | null;
}

export interface LelyConfigResponse {
	enabled: boolean;
	base_url: string;
	username: string;
	farm_key_set: boolean;
	sync_interval_secs: number;
}

export async function getLelyStatus(signal?: AbortSignal) {
	const res = await api<{ data: LelySyncStatus[] }>('/lely/status', { signal });
	return res.data;
}

export async function triggerLelySync() {
	return post<{ message: string }>('/lely/sync', {});
}

export async function getLelyConfig(signal?: AbortSignal) {
	return api<LelyConfigResponse>('/lely/config', { signal });
}
