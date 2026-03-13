import { writable } from 'svelte/store';
import { browser } from '$app/environment';

const DEFAULT_GATEWAY_URL = 'http://localhost:7777';
const STORAGE_KEY = 'agentix-gateway-url';

function createGatewayUrlStore() {
	const initial = browser
		? (localStorage.getItem(STORAGE_KEY) ?? DEFAULT_GATEWAY_URL)
		: DEFAULT_GATEWAY_URL;

	const { subscribe, set } = writable<string>(initial);

	return {
		subscribe,
		set(url: string) {
			set(url);
			if (browser) {
				localStorage.setItem(STORAGE_KEY, url);
			}
		}
	};
}

export const gatewayUrl = createGatewayUrlStore();

/**
 * Update the gateway URL and persist to localStorage.
 */
export function setGatewayUrl(url: string): void {
	gatewayUrl.set(url);
}
