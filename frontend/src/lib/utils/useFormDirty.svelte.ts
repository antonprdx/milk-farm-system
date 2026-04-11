export function useFormDirty() {
	let dirty = $state(false);
	let guardActive = $state(false);

	function enable() {
		if (!guardActive) {
			guardActive = true;
			window.addEventListener('beforeunload', handler);
		}
	}

	function disable() {
		if (guardActive) {
			guardActive = false;
			window.removeEventListener('beforeunload', handler);
		}
	}

	function handler(e: BeforeUnloadEvent) {
		if (dirty) {
			e.preventDefault();
		}
	}

	function markDirty() {
		dirty = true;
		enable();
	}

	function reset() {
		dirty = false;
		disable();
	}

	return {
		get dirty() {
			return dirty;
		},
		markDirty,
		reset
	};
}
