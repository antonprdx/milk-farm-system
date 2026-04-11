export function usePaginatedList(options?: { perPage?: number }) {
	let loading = $state(true);
	let error = $state('');
	let total = $state(0);
	let page = $state(1);
	const perPage = options?.perPage ?? 50;
	let abortController: AbortController | null = null;

	let fromDate = $state('');
	let tillDate = $state('');
	let animalId = $state('');

	function filterParams(): Record<string, string | number | undefined> {
		return {
			from_date: fromDate || undefined,
			till_date: tillDate || undefined,
			animal_id: animalId || undefined,
		};
	}

	async function load<T>(
		fetchFn: (signal?: AbortSignal) => Promise<{ data: T[]; total: number }>,
		onItems: (items: T[]) => void,
		dt?: { setHasRows: (v: boolean) => void },
	) {
		abortController?.abort();
		const controller = new AbortController();
		abortController = controller;
		try {
			loading = true;
			error = '';
			const res = await fetchFn(controller.signal);
			if (controller.signal.aborted) return;
			onItems(res.data);
			total = res.total;
			dt?.setHasRows(res.data.length > 0);
		} catch (e) {
			if (controller.signal.aborted) return;
			error = e instanceof Error ? e.message : 'Ошибка загрузки';
		} finally {
			if (!controller.signal.aborted) {
				loading = false;
			}
		}
	}

	function resetPage() {
		page = 1;
	}

	return {
		get loading() {
			return loading;
		},
		get error() {
			return error;
		},
		set error(v: string) {
			error = v;
		},
		get total() {
			return total;
		},
		get page() {
			return page;
		},
		set page(v: number) {
			page = v;
		},
		perPage,
		get fromDate() {
			return fromDate;
		},
		set fromDate(v: string) {
			fromDate = v;
		},
		get tillDate() {
			return tillDate;
		},
		set tillDate(v: string) {
			tillDate = v;
		},
		get animalId() {
			return animalId;
		},
		set animalId(v: string) {
			animalId = v;
		},
		filterParams,
		load,
		resetPage,
	};
}
