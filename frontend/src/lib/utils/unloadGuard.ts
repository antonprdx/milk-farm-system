import { browser } from '$app/environment';

let active = false;

function handler(e: BeforeUnloadEvent) {
	e.preventDefault();
}

export function enableUnloadGuard() {
	if (browser && !active) {
		active = true;
		window.addEventListener('beforeunload', handler);
	}
}

export function disableUnloadGuard() {
	if (browser && active) {
		active = false;
		window.removeEventListener('beforeunload', handler);
	}
}
