import { api, post, put, del, buildQuery } from './client';

export interface InventoryItem {
	id: number;
	name: string;
	category: string;
	unit: string;
	quantity: number;
	min_quantity: number;
	cost_per_unit: number | null;
	supplier: string | null;
	notes: string | null;
	created_at: string;
	updated_at: string;
}

export interface CreateInventoryItem {
	name: string;
	category: string;
	unit?: string;
	quantity?: number;
	min_quantity?: number;
	cost_per_unit?: number;
	supplier?: string;
	notes?: string;
}

export interface UpdateInventoryItem {
	name?: string;
	category?: string;
	unit?: string;
	min_quantity?: number;
	cost_per_unit?: number;
	supplier?: string;
	notes?: string;
}

export interface InventoryTransaction {
	id: number;
	item_id: number;
	transaction_type: string;
	quantity: number;
	notes: string | null;
	transaction_date: string;
	created_at: string;
}

export interface CreateInventoryTransaction {
	item_id: number;
	transaction_type: string;
	quantity: number;
	notes?: string;
	transaction_date?: string;
}

export interface InventoryFilter {
	category?: string;
	low_stock?: boolean;
	search?: string;
	page?: number;
	per_page?: number;
}

export const INVENTORY_CATEGORY_LABELS: Record<string, string> = {
	feed: 'Корма',
	medicine: 'Медикаменты',
	supplies: 'Расходники',
	equipment: 'Оборудование',
	other: 'Прочее',
};

export function listItems(filter: InventoryFilter = {}, signal?: AbortSignal) {
	return api<{ data: InventoryItem[]; total: number; page: number; per_page: number }>(
		`/inventory${buildQuery(filter)}`,
		{ signal },
	);
}

export function createItem(data: CreateInventoryItem) {
	return post<{ data: InventoryItem }>('/inventory', data);
}

export function updateItem(id: number, data: UpdateInventoryItem) {
	return put<{ data: InventoryItem }>(`/inventory/${id}`, data);
}

export function deleteItem(id: number) {
	return del<{ message: string }>(`/inventory/${id}`);
}

export function createTransaction(itemId: number, data: Omit<CreateInventoryTransaction, 'item_id'>) {
	return post<{ data: InventoryTransaction }>(`/inventory/${itemId}/transaction`, { ...data, item_id: itemId });
}

export function getLowStock(signal?: AbortSignal) {
	return api<{ data: InventoryItem[] }>('/inventory/low-stock', { signal });
}
