import { api, post, put, del, buildQuery } from './client';

export type VetRecordType =
	| 'vaccination'
	| 'treatment'
	| 'disease'
	| 'surgery'
	| 'deworming'
	| 'hoof_care'
	| 'examination'
	| 'other';

export type VetRecordStatus = 'planned' | 'in_progress' | 'completed' | 'cancelled';

export interface VetRecord {
	id: number;
	animal_id: number;
	record_type: VetRecordType;
	status: VetRecordStatus;
	event_date: string;
	diagnosis: string | null;
	treatment: string | null;
	medication: string | null;
	dosage: string | null;
	withdrawal_days: number | null;
	withdrawal_end_date: string | null;
	veterinarian: string | null;
	notes: string | null;
	follow_up_date: string | null;
	created_at: string;
	updated_at: string;
}

export interface CreateVetRecord {
	animal_id: number;
	record_type: VetRecordType;
	status?: VetRecordStatus;
	event_date: string;
	diagnosis?: string;
	treatment?: string;
	medication?: string;
	dosage?: string;
	withdrawal_days?: number;
	veterinarian?: string;
	notes?: string;
	follow_up_date?: string;
}

export interface UpdateVetRecord {
	record_type?: VetRecordType;
	status?: VetRecordStatus;
	event_date?: string;
	diagnosis?: string;
	treatment?: string;
	medication?: string;
	dosage?: string;
	withdrawal_days?: number;
	veterinarian?: string;
	notes?: string;
	follow_up_date?: string;
}

export interface VetRecordFilter {
	animal_id?: number;
	record_type?: VetRecordType;
	status?: VetRecordStatus;
	from_date?: string;
	till_date?: string;
	page?: number;
	per_page?: number;
}

export function listVetRecords(filter: VetRecordFilter = {}, signal?: AbortSignal) {
	return api<{ data: VetRecord[]; total: number }>(`/vet/records${buildQuery(filter)}`, { signal });
}

export function getVetRecord(id: number) {
	return api<{ data: VetRecord }>(`/vet/records/${id}`);
}

export function createVetRecord(data: CreateVetRecord) {
	return post<{ data: VetRecord }>('/vet/records', data);
}

export function updateVetRecord(id: number, data: UpdateVetRecord) {
	return put<{ data: VetRecord }>(`/vet/records/${id}`, data);
}

export function deleteVetRecord(id: number) {
	return del<{ message: string }>(`/vet/records/${id}`);
}

export function getUpcomingFollowUps(days = 7) {
	return api<{ data: VetRecord[] }>(`/vet/follow-ups?days=${days}`);
}

export function getActiveWithdrawals() {
	return api<{ data: VetRecord[] }>('/vet/withdrawals');
}

export interface WeightRecord {
	id: number;
	animal_id: number;
	weight_kg: number;
	bcs: number | null;
	measure_date: string;
	notes: string | null;
	created_at: string;
}

export interface CreateWeightRecord {
	animal_id: number;
	weight_kg: number;
	bcs?: number;
	measure_date: string;
	notes?: string;
}

export interface WeightRecordFilter {
	animal_id?: number;
	from_date?: string;
	till_date?: string;
	page?: number;
	per_page?: number;
}

export function listWeightRecords(filter: WeightRecordFilter = {}, signal?: AbortSignal) {
	return api<{ data: WeightRecord[]; total: number }>(`/vet/weights${buildQuery(filter)}`, { signal });
}

export function createWeightRecord(data: CreateWeightRecord) {
	return post<{ data: WeightRecord }>('/vet/weights', data);
}

export function deleteWeightRecord(id: number) {
	return del<{ message: string }>(`/vet/weights/${id}`);
}

export const VET_RECORD_TYPE_LABELS: Record<VetRecordType, string> = {
	vaccination: 'Вакцинация',
	treatment: 'Лечение',
	disease: 'Заболевание',
	surgery: 'Хирургия',
	deworming: 'Дегельминтизация',
	hoof_care: 'Уход за копытами',
	examination: 'Обследование',
	other: 'Другое',
};

export const VET_STATUS_LABELS: Record<VetRecordStatus, string> = {
	planned: 'Запланировано',
	in_progress: 'В процессе',
	completed: 'Завершено',
	cancelled: 'Отменено',
};
