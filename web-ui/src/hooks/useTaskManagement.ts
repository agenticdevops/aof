import { useCallback, useEffect, useRef } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import type { AppDispatch } from '../store';
import {
  setLoading,
  setError,
  setTasks,
  updateTaskLaneOptimistic,
  commitTaskLaneUpdate,
  rollbackTaskLaneUpdate,
  selectTasksByLane,
  selectTasksLoading,
  selectTasksError,
  selectTaskVersion,
} from '../store/tasksSlice';
import type { Task, TaskLane, MoveTaskRequest, MoveTaskResponse } from '../types/tasks';

/**
 * Base API URL for task operations.
 */
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

/**
 * Exponential backoff configuration.
 */
const RETRY_CONFIG = {
  maxRetries: 3,
  baseDelay: 1000, // 1 second
  maxDelay: 8000, // 8 seconds
};

/**
 * Generate unique request ID.
 */
function generateRequestId(): string {
  return `req_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
}

/**
 * Calculate exponential backoff delay.
 */
function getRetryDelay(attempt: number): number {
  const delay = RETRY_CONFIG.baseDelay * Math.pow(2, attempt);
  return Math.min(delay, RETRY_CONFIG.maxDelay);
}

/**
 * Sleep for specified milliseconds.
 */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Hook return type.
 */
export interface UseTaskManagementResult {
  /** Tasks grouped by lane (optimistic state) */
  tasks: ReturnType<typeof selectTasksByLane>;

  /** Loading state */
  loading: boolean;

  /** Error message (if any) */
  error: string | null;

  /** Move task to different lane */
  moveTask: (taskId: string, newLane: TaskLane) => Promise<void>;

  /** Refetch all tasks from server */
  refetchTasks: () => Promise<void>;
}

/**
 * Hook for managing Kanban board tasks with optimistic updates.
 *
 * Features:
 * - Optimistic UI updates (instant visual feedback)
 * - Version-based conflict resolution
 * - Exponential backoff retry on 5xx errors
 * - AbortController cleanup on unmount
 *
 * @example
 * ```tsx
 * const { tasks, loading, error, moveTask, refetchTasks } = useTaskManagement();
 *
 * // Move task
 * await moveTask('task-123', 'in-progress');
 *
 * // Refresh tasks
 * await refetchTasks();
 * ```
 */
export function useTaskManagement(): UseTaskManagementResult {
  const dispatch = useDispatch<AppDispatch>();
  const tasks = useSelector(selectTasksByLane);
  const loading = useSelector(selectTasksLoading);
  const error = useSelector(selectTasksError);

  // Track abort controllers for cleanup
  const abortControllersRef = useRef<Map<string, AbortController>>(new Map());

  /**
   * Fetch all tasks from server.
   */
  const refetchTasks = useCallback(async () => {
    const controller = new AbortController();
    const requestId = generateRequestId();
    abortControllersRef.current.set(requestId, controller);

    dispatch(setLoading(true));
    dispatch(setError(null));

    try {
      const response = await fetch(`${API_BASE_URL}/api/tasks`, {
        signal: controller.signal,
      });

      if (!response.ok) {
        throw new Error(`Failed to fetch tasks: ${response.statusText}`);
      }

      const fetchedTasks: Task[] = await response.json();
      dispatch(setTasks(fetchedTasks));
    } catch (err) {
      if (err instanceof Error && err.name !== 'AbortError') {
        const errorMessage = err.message || 'Failed to load tasks';
        dispatch(setError(errorMessage));
        console.error('Task fetch error:', err);
      }
    } finally {
      abortControllersRef.current.delete(requestId);
    }
  }, [dispatch]);

  /**
   * Move task to different lane with optimistic update and retry logic.
   */
  const moveTask = useCallback(
    async (taskId: string, newLane: TaskLane) => {
      // Get current version from Redux state
      const currentVersion = useSelector(selectTaskVersion(taskId));
      if (currentVersion === undefined) {
        dispatch(setError(`Task ${taskId} not found`));
        return;
      }

      // Find current lane
      let fromLane: TaskLane | undefined;
      for (const lane of Object.keys(tasks) as TaskLane[]) {
        if (tasks[lane].some((t) => t.id === taskId)) {
          fromLane = lane;
          break;
        }
      }

      if (!fromLane) {
        dispatch(setError(`Task ${taskId} not found in any lane`));
        return;
      }

      // Generate request ID
      const requestId = generateRequestId();
      const controller = new AbortController();
      abortControllersRef.current.set(requestId, controller);

      // Optimistic update
      dispatch(
        updateTaskLaneOptimistic({
          taskId,
          fromLane,
          toLane: newLane,
          requestId,
        }),
      );

      // Prepare request payload
      const payload: MoveTaskRequest = {
        taskId,
        newLane,
        version: currentVersion,
      };

      // Retry logic
      let attempt = 0;
      let lastError: Error | null = null;

      while (attempt <= RETRY_CONFIG.maxRetries) {
        try {
          const response = await fetch(`${API_BASE_URL}/api/tasks/move`, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: JSON.stringify(payload),
            signal: controller.signal,
          });

          if (response.ok) {
            // Success - commit optimistic update
            const result: MoveTaskResponse = await response.json();

            if (result.success && result.task) {
              dispatch(
                commitTaskLaneUpdate({
                  requestId,
                  updatedTask: result.task,
                }),
              );

              console.log(
                `Task ${taskId} moved to ${newLane}, version ${currentVersion} → ${result.task.version}`,
              );
            } else {
              throw new Error(result.error || 'Move failed');
            }

            abortControllersRef.current.delete(requestId);
            return;
          }

          // Handle 409 Conflict (version mismatch)
          if (response.status === 409) {
            const result: MoveTaskResponse = await response.json();
            console.warn(
              `Task ${taskId} version conflict: local=${currentVersion}, server=${result.task?.version || 'unknown'}`,
            );

            // Rollback optimistic update
            dispatch(rollbackTaskLaneUpdate({ requestId }));
            dispatch(setError('Task was modified by another user. Changes rolled back.'));

            abortControllersRef.current.delete(requestId);
            return;
          }

          // Handle 5xx errors with retry
          if (response.status >= 500) {
            throw new Error(`Server error: ${response.status} ${response.statusText}`);
          }

          // Other errors (4xx) - don't retry
          throw new Error(`Request failed: ${response.status} ${response.statusText}`);
        } catch (err) {
          if (err instanceof Error) {
            // Abort error - user cancelled
            if (err.name === 'AbortError') {
              dispatch(rollbackTaskLaneUpdate({ requestId }));
              abortControllersRef.current.delete(requestId);
              return;
            }

            lastError = err;

            // Retry on 5xx or network errors
            if (attempt < RETRY_CONFIG.maxRetries) {
              const delay = getRetryDelay(attempt);
              console.log(
                `Task move failed (attempt ${attempt + 1}/${RETRY_CONFIG.maxRetries}), retrying in ${delay}ms...`,
              );
              await sleep(delay);
              attempt++;
              continue;
            }
          }

          // Max retries exceeded - rollback
          dispatch(rollbackTaskLaneUpdate({ requestId }));
          dispatch(
            setError(
              lastError?.message || 'Network error. Task move failed after multiple retries.',
            ),
          );

          abortControllersRef.current.delete(requestId);
          return;
        }
      }
    },
    [dispatch, tasks],
  );

  /**
   * Cleanup abort controllers on unmount.
   */
  useEffect(() => {
    return () => {
      abortControllersRef.current.forEach((controller) => {
        controller.abort();
      });
      abortControllersRef.current.clear();
    };
  }, []);

  return {
    tasks,
    loading,
    error,
    moveTask,
    refetchTasks,
  };
}
