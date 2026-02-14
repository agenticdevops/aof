/**
 * Task types for Kanban board (Phase 4-02).
 */

/**
 * Task interface for Mission Control Kanban board.
 */
export interface Task {
  /** Task unique identifier */
  id: string;

  /** Task title */
  title: string;

  /** Task description */
  description: string;

  /** Kanban lane */
  lane: 'backlog' | 'assigned' | 'in-progress' | 'review' | 'done';

  /** Agent assigned to this task */
  assignedTo?: string;

  /** Task version (for optimistic locking) */
  version: number;

  /** Creation timestamp */
  createdAt: string;

  /** Last update timestamp */
  updatedAt: string;
}
