import { api, post, put, del, buildQuery } from './client';

export interface Calving {
	id: number;
	animal_id: number;
	calving_date: string;
	remarks: string | null;
	lac_number: number | null;
	created_at: string;
}

export interface Insemination {
	id: number;
	animal_id: number;
	insemination_date: string;
	sire_code: string | null;
	insemination_type: string | null;
	charge_number: string | null;
	created_at: string;
}

export interface Pregnancy {
	id: number;
	animal_id: number;
	pregnancy_date: string;
	pregnancy_type: string | null;
	insemination_date: string | null;
	created_at: string;
}

export interface Heat {
	id: number;
	animal_id: number;
	heat_date: string;
	created_at: string;
}

export interface DryOff {
	id: number;
	animal_id: number;
	dry_off_date: string;
	created_at: string;
}

export interface CreateCalving {
	animal_id: number;
	calving_date: string;
	remarks?: string;
	lac_number?: number;
}

export interface CreateInsemination {
	animal_id: number;
	insemination_date: string;
	sire_code?: string;
	insemination_type?: string;
	charge_number?: string;
}

export interface CreatePregnancy {
	animal_id: number;
	pregnancy_date: string;
	pregnancy_type?: string;
	insemination_date?: string;
}

export interface CreateHeat {
	animal_id: number;
	heat_date: string;
}

export interface CreateDryOff {
	animal_id: number;
	dry_off_date: string;
}

export interface UpdateCalving {
	calving_date?: string;
	remarks?: string;
	lac_number?: number;
}

export interface UpdateInsemination {
	insemination_date?: string;
	sire_code?: string;
	insemination_type?: string;
	charge_number?: string;
}

export interface UpdatePregnancy {
	pregnancy_date?: string;
	pregnancy_type?: string;
	insemination_date?: string;
}

export interface UpdateHeat {
	heat_date?: string;
}

export interface UpdateDryOff {
	dry_off_date?: string;
}

export interface ReproductionFilter {
	animal_id?: string;
	from_date?: string;
	till_date?: string;
	page?: number;
	per_page?: number;
}

export function listCalvings(filter: ReproductionFilter = {}, signal?: AbortSignal) {
	return api<{ data: Calving[]; total: number }>(`/reproduction/calvings${buildQuery(filter)}`, { signal });
}

export function createCalving(data: CreateCalving) {
	return post<{ data: Calving }>('/reproduction/calvings', data);
}

export function updateCalving(id: number, data: UpdateCalving) {
	return put<{ data: Calving }>(`/reproduction/calvings/${id}`, data);
}

export function deleteCalving(id: number) {
	return del<{ message: string }>(`/reproduction/calvings/${id}`);
}

export function listInseminations(filter: ReproductionFilter = {}, signal?: AbortSignal) {
	return api<{ data: Insemination[]; total: number }>(
		`/reproduction/inseminations${buildQuery(filter)}`,
		{ signal },
	);
}

export function createInsemination(data: CreateInsemination) {
	return post<{ data: Insemination }>('/reproduction/inseminations', data);
}

export function updateInsemination(id: number, data: UpdateInsemination) {
	return put<{ data: Insemination }>(`/reproduction/inseminations/${id}`, data);
}

export function deleteInsemination(id: number) {
	return del<{ message: string }>(`/reproduction/inseminations/${id}`);
}

export function listPregnancies(filter: ReproductionFilter = {}, signal?: AbortSignal) {
	return api<{ data: Pregnancy[]; total: number }>(
		`/reproduction/pregnancies${buildQuery(filter)}`,
		{ signal },
	);
}

export function createPregnancy(data: CreatePregnancy) {
	return post<{ data: Pregnancy }>('/reproduction/pregnancies', data);
}

export function updatePregnancy(id: number, data: UpdatePregnancy) {
	return put<{ data: Pregnancy }>(`/reproduction/pregnancies/${id}`, data);
}

export function deletePregnancy(id: number) {
	return del<{ message: string }>(`/reproduction/pregnancies/${id}`);
}

export function listHeats(filter: ReproductionFilter = {}, signal?: AbortSignal) {
	return api<{ data: Heat[]; total: number }>(`/reproduction/heats${buildQuery(filter)}`, { signal });
}

export function createHeat(data: CreateHeat) {
	return post<{ data: Heat }>('/reproduction/heats', data);
}

export function updateHeat(id: number, data: UpdateHeat) {
	return put<{ data: Heat }>(`/reproduction/heats/${id}`, data);
}

export function deleteHeat(id: number) {
	return del<{ message: string }>(`/reproduction/heats/${id}`);
}

export function listDryOffs(filter: ReproductionFilter = {}, signal?: AbortSignal) {
	return api<{ data: DryOff[]; total: number }>(`/reproduction/dryoffs${buildQuery(filter)}`, { signal });
}

export function createDryOff(data: CreateDryOff) {
	return post<{ data: DryOff }>('/reproduction/dryoffs', data);
}

export function updateDryOff(id: number, data: UpdateDryOff) {
	return put<{ data: DryOff }>(`/reproduction/dryoffs/${id}`, data);
}

export function deleteDryOff(id: number) {
	return del<{ message: string }>(`/reproduction/dryoffs/${id}`);
}
