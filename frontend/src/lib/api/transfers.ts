import { api, post, put, del, buildQuery } from './client';

export interface Transfer {
	id: number;
	animal_id: number;
	transfer_date: string;
	transfer_type: string;
	reason_id: number | null;
	from_location: string | null;
	to_location: string | null;
	created_at: string;
}

export interface CreateTransfer {
	animal_id: number;
	transfer_date: string;
	transfer_type: string;
	reason_id?: number;
	from_location?: string;
	to_location?: string;
}

export interface UpdateTransfer {
	transfer_date?: string;
	transfer_type?: string;
	reason_id?: number;
	from_location?: string;
	to_location?: string;
}

export interface TransferFilter {
	animal_id?: string;
	transfer_type?: string;
	page?: number;
	per_page?: number;
}

export const TRANSFER_TYPE_LABELS: Record<string, string> = {
	arrival: 'Прибытие',
	departure: 'Выбытие',
	internal: 'Внутреннее',
	bought: 'Покупка',
	sold: 'Продажа',
	died: 'Падёж',
	slaughtered: 'Забой',
};

export function listTransfers(filter: TransferFilter = {}, signal?: AbortSignal) {
	return api<{ data: Transfer[]; total: number }>(`/transfers${buildQuery(filter)}`, { signal });
}

export function createTransfer(data: CreateTransfer) {
	return post<{ data: Transfer }>('/transfers', data);
}

export function updateTransfer(id: number, data: UpdateTransfer) {
	return put<{ data: Transfer }>(`/transfers/${id}`, data);
}

export function deleteTransfer(id: number) {
	return del<{ message: string }>(`/transfers/${id}`);
}
