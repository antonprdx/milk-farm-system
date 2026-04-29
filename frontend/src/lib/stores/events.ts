import { writable } from 'svelte/store';

export interface SseEvent {
    event_type: string;
    data: unknown;
}

export const notifications = writable<SseEvent[]>([]);
export const connected = writable(false);

let es: EventSource | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

export function connectSSE() {
    if (es) return;

    es = new EventSource('/api/v1/events/stream', { withCredentials: true });

    es.onopen = () => {
        connected.set(true);
        if (reconnectTimer) {
            clearTimeout(reconnectTimer);
            reconnectTimer = null;
        }
    };

    es.onerror = () => {
        connected.set(false);
        es?.close();
        es = null;
        reconnectTimer = setTimeout(connectSSE, 5000);
    };

    const eventTypes = ['alert', 'milk_update', 'task', 'health', 'sync'];

    for (const type of eventTypes) {
        es.addEventListener(type, (e) => {
            try {
                const data = JSON.parse(e.data);
                const event: SseEvent = { event_type: type, data };
                notifications.update((list) => [event, ...list].slice(0, 50));
            } catch {
                // ignore malformed events
            }
        });
    }
}

export function disconnectSSE() {
    if (reconnectTimer) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
    }
    es?.close();
    es = null;
    connected.set(false);
}
