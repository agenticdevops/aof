import { createSlice, type PayloadAction } from '@reduxjs/toolkit';
import type { Task, TasksByLane, TaskLane, PendingRequest } from '../types/tasks';
import type { RootState } from './index';

/**
 * Tasks slice state structure.
 */
interface TasksState {
  /** Server truth - confirmed task state */
  tasks: TasksByLane;

  /** Optimistic state - what UI renders during pending updates */
  optimisticTasks: TasksByLane;

  /** Pending requests - tracking in-flight updates */
  pending: Record<string, PendingRequest>;

  /** Loading state */
  loading: boolean;

  /** Error message (if any) */
  error: string | null;
}

/**
 * Initial empty state for all lanes.
 */
const emptyLanes: TasksByLane = {
  backlog: [],
  assigned: [],
  'in-progress': [],
  review: [],
  done: [],
};

/**
 * Initial tasks slice state.
 */
const initialState: TasksState = {
  tasks: { ...emptyLanes },
  optimisticTasks: { ...emptyLanes },
  pending: {},
  loading: false,
  error: null,
};

/**
 * Payload for optimistic task lane update.
 */
interface OptimisticUpdatePayload {
  taskId: string;
  fromLane: TaskLane;
  toLane: TaskLane;
  requestId: string;
}

/**
 * Payload for committing task lane update.
 */
interface CommitUpdatePayload {
  requestId: string;
  updatedTask: Task;
}

/**
 * Payload for rolling back task lane update.
 */
interface RollbackUpdatePayload {
  requestId: string;
}

/**
 * Payload for handling server task update.
 */
interface ServerTaskUpdatePayload {
  task: Task;
}

/**
 * Tasks slice - manages Kanban board state with optimistic updates.
 */
const tasksSlice = createSlice({
  name: 'tasks',
  initialState,
  reducers: {
    /**
     * Set loading state.
     */
    setLoading(state, action: PayloadAction<boolean>) {
      state.loading = action.payload;
    },

    /**
     * Set error message.
     */
    setError(state, action: PayloadAction<string | null>) {
      state.error = action.payload;
    },

    /**
     * Set all tasks (batch load from server).
     */
    setTasks(state, action: PayloadAction<Task[]>) {
      const lanes = { ...emptyLanes };

      action.payload.forEach((task) => {
        lanes[task.lane].push(task);
      });

      state.tasks = lanes;
      state.optimisticTasks = { ...lanes };
      state.loading = false;
      state.error = null;
    },

    /**
     * Optimistically update task lane (immediate UI feedback).
     */
    updateTaskLaneOptimistic(state, action: PayloadAction<OptimisticUpdatePayload>) {
      const { taskId, fromLane, toLane, requestId } = action.payload;

      // Find task in optimistic state
      const taskIndex = state.optimisticTasks[fromLane].findIndex((t) => t.id === taskId);
      if (taskIndex === -1) return;

      const task = state.optimisticTasks[fromLane][taskIndex];

      // Remove from source lane
      state.optimisticTasks[fromLane].splice(taskIndex, 1);

      // Add to destination lane with updated lane field
      state.optimisticTasks[toLane].push({
        ...task,
        lane: toLane,
      });

      // Track pending request (store timestamp for timeout tracking)
      state.pending[requestId] = {
        taskId,
        controller: new AbortController() as any, // AbortController is not serializable, but we track it
        timestamp: Date.now(),
      };
    },

    /**
     * Commit task lane update (server confirmed).
     */
    commitTaskLaneUpdate(state, action: PayloadAction<CommitUpdatePayload>) {
      const { requestId, updatedTask } = action.payload;

      // Remove pending request
      delete state.pending[requestId];

      // Find task in server truth
      const oldLane = Object.keys(state.tasks).find((lane) =>
        state.tasks[lane as TaskLane].some((t) => t.id === updatedTask.id),
      ) as TaskLane | undefined;

      if (oldLane && oldLane !== updatedTask.lane) {
        // Remove from old lane
        const taskIndex = state.tasks[oldLane].findIndex((t) => t.id === updatedTask.id);
        if (taskIndex !== -1) {
          state.tasks[oldLane].splice(taskIndex, 1);
        }
      }

      // Update or add to new lane
      const newLaneIndex = state.tasks[updatedTask.lane].findIndex(
        (t) => t.id === updatedTask.id,
      );
      if (newLaneIndex !== -1) {
        state.tasks[updatedTask.lane][newLaneIndex] = updatedTask;
      } else {
        state.tasks[updatedTask.lane].push(updatedTask);
      }

      // Sync optimistic state with server truth
      state.optimisticTasks = { ...state.tasks };
    },

    /**
     * Rollback task lane update (server rejected or error).
     */
    rollbackTaskLaneUpdate(state, action: PayloadAction<RollbackUpdatePayload>) {
      const { requestId } = action.payload;

      // Remove pending request
      delete state.pending[requestId];

      // Restore optimistic state from server truth
      state.optimisticTasks = { ...state.tasks };
    },

    /**
     * Handle server task update (WebSocket event or polling).
     * Compares version and applies if server version is newer.
     */
    handleServerTaskUpdate(state, action: PayloadAction<ServerTaskUpdatePayload>) {
      const { task: newTask } = action.payload;

      // Find existing task in server truth
      let existingTask: Task | undefined;
      let existingLane: TaskLane | undefined;

      for (const lane of Object.keys(state.tasks) as TaskLane[]) {
        const index = state.tasks[lane].findIndex((t) => t.id === newTask.id);
        if (index !== -1) {
          existingTask = state.tasks[lane][index];
          existingLane = lane;
          break;
        }
      }

      // Version comparison: apply if server version > local version
      if (!existingTask || newTask.version > existingTask.version) {
        // Remove from old lane if exists
        if (existingLane) {
          const index = state.tasks[existingLane].findIndex((t) => t.id === newTask.id);
          if (index !== -1) {
            state.tasks[existingLane].splice(index, 1);
          }
        }

        // Add to new lane
        const newLaneIndex = state.tasks[newTask.lane].findIndex((t) => t.id === newTask.id);
        if (newLaneIndex !== -1) {
          state.tasks[newTask.lane][newLaneIndex] = newTask;
        } else {
          state.tasks[newTask.lane].push(newTask);
        }

        // Only update optimistic if no pending request for this task
        const hasPendingRequest = Object.values(state.pending).some(
          (req) => req.taskId === newTask.id,
        );
        if (!hasPendingRequest) {
          state.optimisticTasks = { ...state.tasks };
        }
      }
    },
  },
});

/**
 * Actions.
 */
export const {
  setLoading,
  setError,
  setTasks,
  updateTaskLaneOptimistic,
  commitTaskLaneUpdate,
  rollbackTaskLaneUpdate,
  handleServerTaskUpdate,
} = tasksSlice.actions;

/**
 * Selectors.
 */

/**
 * Select tasks by lane (returns optimistic state for UI rendering).
 */
export const selectTasksByLane = (state: RootState): TasksByLane => state.tasks.optimisticTasks;

/**
 * Select tasks for a specific lane.
 */
export const selectTasksForLane = (lane: TaskLane) => (state: RootState): Task[] =>
  state.tasks.optimisticTasks[lane];

/**
 * Select task version by ID.
 */
export const selectTaskVersion = (taskId: string) => (state: RootState): number | undefined => {
  for (const lane of Object.keys(state.tasks.tasks) as TaskLane[]) {
    const task = state.tasks.tasks[lane].find((t) => t.id === taskId);
    if (task) return task.version;
  }
  return undefined;
};

/**
 * Select pending requests count.
 */
export const selectPendingCount = (state: RootState): number =>
  Object.keys(state.tasks.pending).length;

/**
 * Select loading state.
 */
export const selectTasksLoading = (state: RootState): boolean => state.tasks.loading;

/**
 * Select error state.
 */
export const selectTasksError = (state: RootState): string | null => state.tasks.error;

/**
 * Default export.
 */
export default tasksSlice.reducer;
