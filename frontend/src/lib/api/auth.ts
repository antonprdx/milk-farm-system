import { post } from './client';

export interface LoginResponse {
	username: string;
	role: string;
	must_change_password: boolean;
}

export async function login(username: string, password: string) {
	return post<LoginResponse>('/auth/login', { username, password });
}

export async function logout() {
	return post<{ message: string }>('/auth/logout', {});
}

export async function refresh() {
	return post<LoginResponse>('/auth/refresh', {});
}
