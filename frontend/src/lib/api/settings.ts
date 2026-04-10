import { api, post, del, put } from '$lib/api/client';

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
