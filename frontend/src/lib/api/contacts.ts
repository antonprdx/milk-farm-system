import { api, post, put, del, buildQuery } from './client';

export interface Contact {
	id: number;
	name: string;
	contact_type_id: number | null;
	contact_type_name: string | null;
	farm_number: string | null;
	phone_cell: string | null;
	phone_home: string | null;
	phone_work: string | null;
	email: string | null;
	company_name: string | null;
	description: string | null;
	active: boolean;
	created_at: string;
}

export interface CreateContact {
	name: string;
	type_id?: number;
	farm_number?: string;
	active?: boolean;
	phone_cell?: string;
	phone_home?: string;
	phone_work?: string;
	email?: string;
	company_name?: string;
	description?: string;
}

export interface UpdateContact {
	name?: string;
	type_id?: number;
	farm_number?: string;
	active?: boolean;
	phone_cell?: string;
	phone_home?: string;
	phone_work?: string;
	email?: string;
	company_name?: string;
	description?: string;
}

export interface ContactFilter {
	page?: number;
	per_page?: number;
}

export function listContacts(filter: ContactFilter = {}) {
	return api<{ data: Contact[]; total: number }>(`/contacts${buildQuery(filter)}`);
}

export function createContact(data: CreateContact) {
	return post<{ data: Contact }>('/contacts', data);
}

export function updateContact(id: number, data: UpdateContact) {
	return put<{ data: Contact }>(`/contacts/${id}`, data);
}

export function deleteContact(id: number) {
	return del<{ message: string }>(`/contacts/${id}`);
}
