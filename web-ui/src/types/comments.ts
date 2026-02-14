/**
 * TypeScript types for task comments.
 */

/**
 * Task comment interface.
 */
export interface TaskComment {
  /** Unique comment identifier */
  id: string;

  /** Task ID this comment belongs to */
  taskId: string;

  /** Author ID */
  authorId: string;

  /** Author display name */
  authorName: string;

  /** Author avatar (emoji or URL) */
  authorAvatar?: string;

  /** Comment text (supports markdown) */
  text: string;

  /** ISO 8601 timestamp when comment was created */
  timestamp: string;

  /** Optional thread/reply information */
  threadId?: string;
  replyTo?: string;

  /** Version number for conflict resolution */
  version?: number;
}

/**
 * Generate optimistic comment ID.
 */
export function generateOptimisticCommentId(): string {
  return `temp_comment_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
}
