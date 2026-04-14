import { api, post, put, del, buildQuery } from './client';

export interface Sire {
	id: number;
	sire_code: string | null;
	life_number: string | null;
	name: string | null;
	active: boolean | null;
	created_at: string;
}

export interface CreateSire {
	sire_code?: string;
	life_number?: string;
	name?: string;
	active?: boolean;
}

export interface UpdateSire {
	sire_code?: string;
	life_number?: string;
	name?: string;
	active?: boolean;
}

export interface SireFilter {
	search?: string;
	page?: number;
	per_page?: number;
}

export const SIRE_LABELS: Record<string, string> = {
	sire_code: 'Код быка',
	life_number: 'Жизненный номер',
	name: 'Имя',
	active: 'Активен',
};

export function listSires(filter: SireFilter = {}, signal?: AbortSignal) {
	return api<{ data: Sire[]; total: number }>(`/sires${buildQuery(filter)}`, { signal });
}

export function createSire(data: CreateSire) {
	return post<{ data: Sire }>('/sires', data);
}

export function updateSire(id: number, data: UpdateSire) {
	return put<{ data: Sire }>(`/sires/${id}`, data);
}

export function deleteSire(id: number) {
	return del<{ message: string }>(`/sires/${id}`);
}
