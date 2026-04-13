import { api, post, put, del, buildQuery } from './client';

export interface Location {
	id: number;
	name: string;
	location_type: string | null;
	created_at: string;
}

export interface CreateLocation {
	name: string;
	location_type?: string | null;
}

export interface UpdateLocation {
	name?: string;
	location_type?: string | null;
}

export interface LocationFilter {
	page?: number;
	per_page?: number;
}

export function listLocations(filter: LocationFilter = {}, signal?: AbortSignal) {
	return api<{ data: Location[]; total: number }>(`/locations${buildQuery(filter)}`, { signal });
}

export function createLocation(data: CreateLocation) {
	return post<{ data: Location }>('/locations', data);
}

export function updateLocation(id: number, data: UpdateLocation) {
	return put<{ data: Location }>(`/locations/${id}`, data);
}

export function deleteLocation(id: number) {
	return del(`/locations/${id}`);
}
