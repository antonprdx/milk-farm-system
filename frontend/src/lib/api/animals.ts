import { api, del, post, put, buildQuery } from './client';

export interface Animal {
	id: number;
	life_number: string | null;
	name: string | null;
	user_number: number | null;
	gender: 'male' | 'female';
	birth_date: string;
	hair_color_code: string | null;
	father_life_number: string | null;
	mother_life_number: string | null;
	description: string | null;
	ucn_number: string | null;
	use_as_sire: boolean | null;
	location: string | null;
	group_number: number | null;
	keep: boolean | null;
	gestation: number | null;
	responder_number: string | null;
	active: boolean;
	created_at: string;
	updated_at: string;
}

export interface AnimalListResponse {
	data: Animal[];
	total: number;
}

export interface AnimalFilter {
	life_number?: string;
	ucn_number?: string;
	active?: boolean;
	gender?: 'male' | 'female';
	page?: number;
	per_page?: number;
}

export interface CreateAnimal {
	life_number?: string;
	name?: string;
	user_number?: number;
	gender: 'male' | 'female';
	birth_date: string;
	hair_color_code?: string;
	father_life_number?: string;
	mother_life_number?: string;
	description?: string;
	ucn_number?: string;
	use_as_sire?: boolean;
	location?: string;
	group_number?: number;
	keep?: boolean;
	gestation?: number;
	responder_number?: string;
}

export interface UpdateAnimal {
	name?: string;
	hair_color_code?: string;
	description?: string;
	ucn_number?: string;
	use_as_sire?: boolean;
	location?: string;
	group_number?: number;
	keep?: boolean;
	gestation?: number;
	responder_number?: string;
	active?: boolean;
}

export function listAnimals(filter: AnimalFilter = {}) {
	return api<AnimalListResponse>(`/animals${buildQuery(filter)}`);
}

export function getAnimal(id: number) {
	return api<{ data: Animal }>(`/animals/${id}`);
}

export function createAnimal(data: CreateAnimal) {
	return post<{ data: Animal }>('/animals', data);
}

export function updateAnimal(id: number, data: UpdateAnimal) {
	return put<{ data: Animal }>(`/animals/${id}`, data);
}

export function deleteAnimal(id: number) {
	return del<{ message: string }>(`/animals/${id}`);
}
