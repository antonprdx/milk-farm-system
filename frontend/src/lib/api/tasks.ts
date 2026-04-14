import { api, post, put, del, buildQuery } from './client';

export type TaskStatus = 'pending' | 'in_progress' | 'done' | 'cancelled';
export type TaskPriority = 'low' | 'medium' | 'high' | 'urgent';
export type TaskCategory = 'health' | 'reproduction' | 'feeding' | 'maintenance' | 'administrative' | 'other';

export interface Task {
	id: number;
	title: string;
	description: string | null;
	category: TaskCategory;
	priority: TaskPriority;
	status: TaskStatus;
	animal_id: number | null;
	due_date: string | null;
	assigned_to: string | null;
	completed_at: string | null;
	created_by: number | null;
	created_at: string;
	updated_at: string;
}

export interface CreateTask {
	title: string;
	description?: string;
	category?: TaskCategory;
	priority?: TaskPriority;
	animal_id?: number;
	due_date?: string;
	assigned_to?: string;
}

export interface UpdateTask {
	title?: string;
	description?: string;
	category?: TaskCategory;
	priority?: TaskPriority;
	status?: TaskStatus;
	animal_id?: number;
	due_date?: string;
	assigned_to?: string;
}

export interface TaskFilter {
	status?: TaskStatus;
	priority?: TaskPriority;
	category?: TaskCategory;
	animal_id?: number;
	overdue?: boolean;
	from_date?: string;
	till_date?: string;
	page?: number;
	per_page?: number;
}

export function listTasks(filter: TaskFilter = {}, signal?: AbortSignal) {
	return api<{ data: Task[]; total: number }>(`/tasks${buildQuery(filter)}`, { signal });
}
export function createTask(data: CreateTask) { return post<{ data: Task }>('/tasks', data); }
export function updateTask(id: number, data: UpdateTask) { return put<{ data: Task }>(`/tasks/${id}`, data); }
export function deleteTask(id: number) { return del<{ message: string }>(`/tasks/${id}`); }

export interface Transaction {
	id: number;
	transaction_type: string;
	category: string;
	amount: number;
	description: string | null;
	transaction_date: string;
	animal_id: number | null;
	reference: string | null;
	created_at: string;
	updated_at: string;
}

export interface CreateTransaction {
	transaction_type: string;
	category: string;
	amount: number;
	description?: string;
	transaction_date: string;
	animal_id?: number;
	reference?: string;
}

export interface TransactionFilter {
	transaction_type?: string;
	category?: string;
	from_date?: string;
	till_date?: string;
	animal_id?: number;
	page?: number;
	per_page?: number;
}

export function listTransactions(filter: TransactionFilter = {}, signal?: AbortSignal) {
	return api<{ data: Transaction[]; total: number }>(`/finance${buildQuery(filter)}`, { signal });
}
export function createTransaction(data: CreateTransaction) { return post<{ data: Transaction }>('/finance', data); }
export function deleteTransaction(id: number) { return del<{ message: string }>(`/finance/${id}`); }

export interface AuditLogEntry {
	id: number;
	user_id: number | null;
	action: string;
	entity_type: string;
	entity_id: number | null;
	details: any;
	created_at: string;
}

export interface AuditLogFilter {
	user_id?: number;
	entity_type?: string;
	action?: string;
	from_date?: string;
	page?: number;
	per_page?: number;
}

export function listAuditLog(filter: AuditLogFilter = {}) {
	return api<{ data: AuditLogEntry[]; total: number }>(`/audit-log${buildQuery(filter)}`);
}

export interface SearchResult {
	animals: { id: number; name: string | null; life_number: string | null }[];
	contacts: { id: number; name: string; company_name: string | null }[];
}

export function globalSearch(q: string) {
	return api<SearchResult>(`/search?q=${encodeURIComponent(q)}`);
}

export const TASK_STATUS_LABELS: Record<TaskStatus, string> = {
	pending: 'Ожидает',
	in_progress: 'В процессе',
	done: 'Выполнено',
	cancelled: 'Отменено',
};

export const TASK_PRIORITY_LABELS: Record<TaskPriority, string> = {
	low: 'Низкий',
	medium: 'Средний',
	high: 'Высокий',
	urgent: 'Срочный',
};

export const TASK_CATEGORY_LABELS: Record<TaskCategory, string> = {
	health: 'Здоровье',
	reproduction: 'Воспроизводство',
	feeding: 'Кормление',
	maintenance: 'Обслуживание',
	administrative: 'Административное',
	other: 'Другое',
};

export const FINANCE_CATEGORIES = {
	income: ['milk_sales', 'animal_sales', 'subsidies', 'other_income'],
	expense: ['feed', 'medicine', 'vet_services', 'equipment', 'labor', 'utilities', 'other_expense'],
} as const;

export const FINANCE_CATEGORY_LABELS: Record<string, string> = {
	milk_sales: 'Продажа молока',
	animal_sales: 'Продажа животных',
	subsidies: 'Субсидии',
	other_income: 'Другой доход',
	feed: 'Корма',
	medicine: 'Медикаменты',
	vet_services: 'Вет. услуги',
	equipment: 'Оборудование',
	labor: 'Зарплата',
	utilities: 'Коммунальные',
	other_expense: 'Другой расход',
};
