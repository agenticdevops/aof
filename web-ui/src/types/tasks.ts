/**
 * Task types for Kanban board (Phase 4-02).
 */

/**
 * Task lane identifiers.
 * Tasks flow through lanes from left to right: backlog → assigned → in-progress → review → done
 */
export type TaskLane = 'backlog' | 'assigned' | 'in-progress' | 'review' | 'done';

/**
 * Task status indicators.
 */
export type TaskStatus = 'pending' | 'active' | 'blocked' | 'completed' | 'cancelled';

/**
 * Task priority levels.
 */
export type TaskPriority = 'low' | 'medium' | 'high' | 'critical';

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
  lane: TaskLane;

  /** Agent assigned to this task */
  assignedTo?: string;

  /** Task version (for optimistic locking) */
  version: number;

  /** Creation timestamp */
  createdAt: string;

  /** Last update timestamp */
  updatedAt: string;

  /** Current task status */
  status: TaskStatus;

  /** Task priority */
  priority?: TaskPriority;

  /** Tags/labels for categorization */
  tags?: string[];

  /** Due date (ISO 8601) (optional) */
  dueDate?: string;
}

/**
 * Request payload for moving a task to a different lane.
 */
export interface MoveTaskRequest {
  /** Task to move */
  taskId: string;

  /** Destination lane */
  newLane: TaskLane;

  /** Current version (for optimistic concurrency) */
  version: number;
}

/**
 * Response from task move operation.
 */
export interface MoveTaskResponse {
  /** Updated task with new version */
  task: Task;

  /** Success flag */
  success: boolean;

  /** Error message (if success=false) */
  error?: string;
}

/**
 * Tasks grouped by lane.
 */
export type TasksByLane = Record<TaskLane, Task[]>;

/**
 * Pending request tracking.
 */
export interface PendingRequest {
  taskId: string;
  controller: AbortController;
  timestamp: number;
}
