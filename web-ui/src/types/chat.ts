/**
 * TypeScript types for chat functionality.
 */

/**
 * Chat message interface.
 */
export interface ChatMessage {
  /** Unique message identifier (from server or optimistic temp ID) */
  id: string;

  /** ID of the sender (agent or user) */
  senderId: string;

  /** Display name of sender */
  senderName: string;

  /** Avatar URL or emoji */
  senderAvatar?: string;

  /** Message content (supports markdown) */
  content: string;

  /** ISO 8601 timestamp when message was sent */
  timestamp: string;

  /** Optional thread ID for reply threading */
  threadId?: string;

  /** Version number for conflict resolution */
  version?: number;
}

/**
 * Generate optimistic message ID (client-side temporary ID).
 */
export function generateOptimisticId(): string {
  return `temp_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
}
