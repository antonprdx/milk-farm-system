import { api, post, put, del, buildQuery } from './client';

export interface BulkTankTest {
	id: number;
	date: string;
	fat: number;
	protein: number;
	lactose: number | null;
	scc: number | null;
	ffa: number | null;
}

export interface CreateBulkTankTest {
	date: string;
	fat: number;
	protein: number;
	lactose?: number;
	scc?: number;
	ffa?: number;
}

export interface UpdateBulkTankTest {
	date?: string;
	fat?: number;
	protein?: number;
	lactose?: number;
	scc?: number;
	ffa?: number;
}

export interface BulkTankFilter {
	from_date?: string;
	till_date?: string;
	page?: number;
	per_page?: number;
}

export function listBulkTankTests(filter: BulkTankFilter = {}) {
	return api<{ data: BulkTankTest[]; total: number }>(`/bulk-tank${buildQuery(filter)}`);
}

export function createBulkTankTest(data: CreateBulkTankTest) {
	return post<{ data: BulkTankTest }>('/bulk-tank', data);
}

export function updateBulkTankTest(id: number, data: UpdateBulkTankTest) {
	return put<{ data: BulkTankTest }>(`/bulk-tank/${id}`, data);
}

export function deleteBulkTankTest(id: number) {
	return del<{ message: string }>(`/bulk-tank/${id}`);
}
