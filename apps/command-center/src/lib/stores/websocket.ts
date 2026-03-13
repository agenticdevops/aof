import { writable, readable, get } from 'svelte/store';
import { browser } from '$app/environment';
import { gatewayUrl } from '$lib/stores/gateway.js';

// ============================================================
// GatewayEvent types (mirror Rust enum GatewayEvent)
// ============================================================

export interface AgentStatusEvent {
	type: 'agent_status';
	agent_name: string;
	status: 'running' | 'idle' | 'error' | 'scheduled';
}

export interface RunStartedEvent {
	type: 'run_started';
	agent_name: string;
	run_id: string;
	trigger_source: string;
}

export interface RunCompletedEvent {
	type: 'run_completed';
	agent_name: string;
	run_id: string;
	status: 'completed' | 'failed' | 'stopped';
	duration_ms?: number;
	cost_usd?: number;
}

export interface ApprovalRequestedEvent {
	type: 'approval_requested';
	id: string;
	agent_name: string;
	run_id: string;
	action_description: string;
}

export interface ApprovalDecidedEvent {
	type: 'approval_decided';
	id: string;
	agent_name: string;
	decision: 'approved' | 'denied';
}

export interface CostUpdateEvent {
	type: 'cost_update';
	agent_name: string;
	cost_usd: number;
	total_today_usd: number;
}

export interface ConnectedEvent {
	type: 'connected';
}

export type GatewayEvent =
	| AgentStatusEvent
	| RunStartedEvent
	| RunCompletedEvent
	| ApprovalRequestedEvent
	| ApprovalDecidedEvent
	| CostUpdateEvent
	| ConnectedEvent;

// ============================================================
// Connection status
// ============================================================

export type WsStatus = 'connecting' | 'connected' | 'disconnected';

// ============================================================
// Internal state
// ============================================================

let socket: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let reconnectDelay = 1000; // ms, doubles each attempt up to MAX_DELAY
const MAX_RECONNECT_DELAY = 30_000;
let intentionalDisconnect = false;

// ============================================================
// Stores
// ============================================================

const _wsStatus = writable<WsStatus>('disconnected');
export const wsStatus = { subscribe: _wsStatus.subscribe };

const _wsEvents = writable<GatewayEvent | null>(null);
export const wsEvents = { subscribe: _wsEvents.subscribe };

// ============================================================
// WebSocket lifecycle
// ============================================================

function buildWsUrl(): string {
	const url = get(gatewayUrl).replace(/\/$/, '');
	// Convert http:// -> ws:// and https:// -> wss://
	return url.replace(/^http:/, 'ws:').replace(/^https:/, 'wss:') + '/ws';
}

function scheduleReconnect() {
	if (intentionalDisconnect) return;

	reconnectTimer = setTimeout(() => {
		reconnectTimer = null;
		connect();
	}, reconnectDelay);

	// Exponential backoff
	reconnectDelay = Math.min(reconnectDelay * 2, MAX_RECONNECT_DELAY);
}

function connect() {
	if (!browser) return;
	if (socket && (socket.readyState === WebSocket.CONNECTING || socket.readyState === WebSocket.OPEN)) {
		return;
	}

	_wsStatus.set('connecting');

	try {
		const wsUrl = buildWsUrl();
		socket = new WebSocket(wsUrl);

		socket.onopen = () => {
			reconnectDelay = 1000; // reset backoff on successful connection
			_wsStatus.set('connected');
		};

		socket.onmessage = (event: MessageEvent) => {
			try {
				const parsed = JSON.parse(event.data as string) as GatewayEvent;
				_wsEvents.set(parsed);
			} catch {
				// Ignore malformed frames
			}
		};

		socket.onclose = () => {
			socket = null;
			_wsStatus.set('disconnected');
			scheduleReconnect();
		};

		socket.onerror = () => {
			// onclose will fire after onerror — reconnect handled there
			socket = null;
			_wsStatus.set('disconnected');
		};
	} catch {
		_wsStatus.set('disconnected');
		scheduleReconnect();
	}
}

// ============================================================
// Public API
// ============================================================

/**
 * Initialize the WebSocket connection to the gateway.
 * Call on layout mount. Auto-reconnects on disconnect.
 */
export function initWebSocket(): void {
	if (!browser) return;
	intentionalDisconnect = false;
	connect();
}

/**
 * Tear down the WebSocket connection. Call on layout destroy.
 */
export function disconnectWebSocket(): void {
	intentionalDisconnect = true;
	if (reconnectTimer !== null) {
		clearTimeout(reconnectTimer);
		reconnectTimer = null;
	}
	if (socket) {
		socket.close();
		socket = null;
	}
	_wsStatus.set('disconnected');
}
