import { writable, derived, get } from 'svelte/store';
import { api } from '$lib/api/client.js';
import { wsEvents } from '$lib/stores/websocket.js';
import type { ApprovalRequest } from '$lib/api/types.js';

// ============================================================
// Stores
// ============================================================

export const approvals = writable<ApprovalRequest[]>([]);
export const approvalsLoading = writable<boolean>(false);
export const approvalsError = writable<string | null>(null);

// ============================================================
// Derived stores
// ============================================================

/** All pending approval requests (status === 'pending'). */
export const pendingApprovals = derived(approvals, ($approvals) =>
	$approvals.filter((r) => r.status === 'pending')
);

/** All decided approval requests (approved, denied, expired). */
export const decidedApprovals = derived(approvals, ($approvals) =>
	$approvals.filter((r) => r.status !== 'pending')
);

// ============================================================
// Actions
// ============================================================

/**
 * Fetch all approval requests from the gateway REST API and update the store.
 */
export async function loadApprovals(): Promise<void> {
	approvalsLoading.set(true);
	approvalsError.set(null);
	try {
		const list = await api.approvals.list();
		approvals.set(list);
	} catch (err) {
		approvalsError.set(err instanceof Error ? err.message : 'Failed to load approvals');
	} finally {
		approvalsLoading.set(false);
	}
}

/**
 * Approve a pending request with optimistic UI update.
 * Immediately marks status='approved' in the store; reverts on API failure.
 * Returns true on success, false on failure.
 */
export async function approveRequest(id: string): Promise<boolean> {
	// Optimistic update: mark as approved immediately
	const previous = get(approvals);
	approvals.update((list) =>
		list.map((r) =>
			r.id === id
				? { ...r, status: 'approved' as const, decided_at: new Date().toISOString() }
				: r
		)
	);

	try {
		await api.approvals.approve(id);
		return true;
	} catch (err) {
		// Revert to previous state on failure
		approvals.set(previous);
		approvalsError.set(err instanceof Error ? err.message : 'Failed to approve request');
		return false;
	}
}

/**
 * Deny a pending request with optional reason and optimistic UI update.
 * Immediately marks status='denied' in the store; reverts on API failure.
 * Returns true on success, false on failure.
 */
export async function denyRequest(id: string, reason?: string): Promise<boolean> {
	// Optimistic update: mark as denied immediately
	const previous = get(approvals);
	approvals.update((list) =>
		list.map((r) =>
			r.id === id
				? { ...r, status: 'denied' as const, decided_at: new Date().toISOString() }
				: r
		)
	);

	try {
		await api.approvals.deny(id, undefined, reason);
		return true;
	} catch (err) {
		// Revert to previous state on failure
		approvals.set(previous);
		approvalsError.set(err instanceof Error ? err.message : 'Failed to deny request');
		return false;
	}
}

// ============================================================
// WebSocket reactive updates
// ============================================================

wsEvents.subscribe((event) => {
	if (!event) return;

	if (event.type === 'approval_requested') {
		// Check if this request already exists in the store
		const existing = get(approvals).find((r) => r.id === event.id);
		if (!existing) {
			// Add the new pending request at the top of the list
			const newRequest: ApprovalRequest = {
				id: event.id,
				agent_name: event.agent_name,
				run_id: event.run_id,
				action_description: event.action_description,
				status: 'pending',
				requested_at: new Date().toISOString()
			};
			approvals.update((list) => [newRequest, ...list]);
		}
	} else if (event.type === 'approval_decided') {
		// Update the matching request's status in-place
		approvals.update((list) =>
			list.map((r) =>
				r.id === event.id
					? {
							...r,
							status: event.decision,
							decided_at: r.decided_at ?? new Date().toISOString()
						}
					: r
			)
		);
	}
});
