import { api, post, put } from './client';

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
	password_set: boolean;
	farm_key_set: boolean;
	sync_interval_secs: number;
}

export interface UpdateLelyConfigRequest {
	enabled?: boolean;
	base_url?: string;
	username?: string;
	password?: string;
	farm_key?: string;
	sync_interval_secs?: number;
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

export async function updateLelyConfig(data: UpdateLelyConfigRequest) {
	return put<{ message: string }>('/lely/config', data);
}

export async function testLelyConnection(data: UpdateLelyConfigRequest) {
	return post<{ success: boolean; message: string }>('/lely/test-connection', data);
}
