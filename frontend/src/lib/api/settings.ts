import { api, post, del, put } from './client';

export interface UserItem {
	id: number;
	username: string;
	role: string;
	created_at: string;
}

export interface ListUsersResponse {
	data: UserItem[];
}

export interface CreateUserRequest {
	username: string;
	password: string;
}

export interface ChangePasswordRequest {
	old_password: string;
	new_password: string;
}

export interface UserPreferences {
	theme: string;
	page_size: number;
	compact_view: boolean;
	language: string;
}

export interface SystemInfo {
	version: string;
	uptime_secs: number;
	db_size_mb: number;
	total_animals: number;
	total_milk_records: number;
	total_reproduction_records: number;
	total_users: number;
}

export interface JwtTtlSettings {
	jwt_access_ttl_secs: number;
	jwt_refresh_ttl_secs: number;
}

export interface AlertThresholds {
	alert_min_milk: number;
	alert_max_scc: number;
	alert_days_before_calving: number;
	alert_activity_drop_pct: number;
}

export async function listUsers() {
	return api<ListUsersResponse>('/settings/users');
}

export async function createUser(data: CreateUserRequest) {
	return post<{ message: string }>('/settings/users', data);
}

export async function changePassword(data: ChangePasswordRequest) {
	return post<{ message: string }>('/settings/password', data);
}

export async function deleteUser(id: number) {
	return del<{ message: string }>(`/settings/users/${id}`);
}

export async function updateUserRole(id: number, role: string) {
	return put<{ message: string }>(`/settings/users/${id}/role`, { role });
}

export async function getPreferences() {
	return api<UserPreferences>('/settings/preferences');
}

export async function updatePreferences(data: Partial<UserPreferences>) {
	return put<UserPreferences>('/settings/preferences', data);
}

export async function getSystemInfo() {
	return api<SystemInfo>('/settings/system-info');
}

export async function getJwtTtl() {
	return api<JwtTtlSettings>('/settings/jwt-ttl');
}

export async function updateJwtTtl(data: Partial<JwtTtlSettings>) {
	return put<JwtTtlSettings>('/settings/jwt-ttl', data);
}

export async function getAlertThresholds() {
	return api<AlertThresholds>('/settings/alert-thresholds');
}

export async function updateAlertThresholds(data: Partial<AlertThresholds>) {
	return put<AlertThresholds>('/settings/alert-thresholds', data);
}

export function getBackupUrl() {
	const base = import.meta.env.VITE_API_BASE || '/api/v1';
	return `${base}/settings/backup`;
}
