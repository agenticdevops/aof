/**
 * Unit tests for tasksSlice - version-based conflict resolution.
 */

import { describe, it, expect } from 'vitest';
import tasksReducer, {
  setTasks,
  updateTaskLaneOptimistic,
  commitTaskLaneUpdate,
  rollbackTaskLaneUpdate,
  handleServerTaskUpdate,
} from './tasksSlice';
import type { Task } from '../types/tasks';

describe('tasksSlice', () => {
  const mockTask: Task = {
    id: 'task-1',
    title: 'Test task',
    description: 'Test description',
    lane: 'backlog',
    version: 3,
    status: 'pending',
    createdAt: '2024-02-14T10:00:00Z',
    updatedAt: '2024-02-14T10:00:00Z',
  };

  describe('version-based conflict resolution', () => {
    it('should apply server update if server version > local version', () => {
      // Initial state: task with version 3 in backlog
      const initialState = tasksReducer(
        undefined,
        setTasks([mockTask]),
      );

      expect(initialState.tasks.backlog).toHaveLength(1);
      expect(initialState.tasks.backlog[0].version).toBe(3);

      // Server sends update with version 5 (moved to in-progress)
      const serverTask: Task = {
        ...mockTask,
        lane: 'in-progress',
        version: 5,
        updatedAt: '2024-02-14T12:00:00Z',
      };

      const newState = tasksReducer(
        initialState,
        handleServerTaskUpdate({ task: serverTask }),
      );

      // Verify version 5 applied
      expect(newState.tasks.backlog).toHaveLength(0);
      expect(newState.tasks['in-progress']).toHaveLength(1);
      expect(newState.tasks['in-progress'][0].version).toBe(5);
      expect(newState.tasks['in-progress'][0].lane).toBe('in-progress');
    });

    it('should ignore server update if server version <= local version', () => {
      // Initial state: task with version 5 in in-progress
      const taskV5: Task = { ...mockTask, version: 5, lane: 'in-progress' };
      const initialState = tasksReducer(
        undefined,
        setTasks([taskV5]),
      );

      expect(initialState.tasks['in-progress']).toHaveLength(1);
      expect(initialState.tasks['in-progress'][0].version).toBe(5);

      // Server sends update with version 3 (older)
      const serverTask: Task = {
        ...mockTask,
        lane: 'backlog',
        version: 3,
        updatedAt: '2024-02-14T09:00:00Z',
      };

      const newState = tasksReducer(
        initialState,
        handleServerTaskUpdate({ task: serverTask }),
      );

      // Verify version 5 retained (server update ignored)
      expect(newState.tasks['in-progress']).toHaveLength(1);
      expect(newState.tasks['in-progress'][0].version).toBe(5);
      expect(newState.tasks['in-progress'][0].lane).toBe('in-progress');
      expect(newState.tasks.backlog).toHaveLength(0);
    });

    it('should not update optimistic state if pending request exists', () => {
      // Initial state: task with version 3 in backlog
      const initialState = tasksReducer(
        undefined,
        setTasks([mockTask]),
      );

      // Optimistic update (task moving to in-progress)
      const requestId = 'req_123';
      const afterOptimistic = tasksReducer(
        initialState,
        updateTaskLaneOptimistic({
          taskId: 'task-1',
          fromLane: 'backlog',
          toLane: 'in-progress',
          requestId,
        }),
      );

      // Verify optimistic state updated
      expect(afterOptimistic.optimisticTasks['in-progress']).toHaveLength(1);
      expect(afterOptimistic.optimisticTasks.backlog).toHaveLength(0);

      // Server sends update with version 4 while request is pending
      const serverTask: Task = {
        ...mockTask,
        lane: 'review',
        version: 4,
        updatedAt: '2024-02-14T11:00:00Z',
      };

      const afterServerUpdate = tasksReducer(
        afterOptimistic,
        handleServerTaskUpdate({ task: serverTask }),
      );

      // Verify server truth updated, but optimistic state unchanged (pending request exists)
      expect(afterServerUpdate.tasks.review).toHaveLength(1);
      expect(afterServerUpdate.tasks.review[0].version).toBe(4);
      expect(afterServerUpdate.optimisticTasks['in-progress']).toHaveLength(1); // Still in optimistic lane
      expect(afterServerUpdate.optimisticTasks.review).toHaveLength(0); // Not in optimistic state yet
    });
  });

  describe('optimistic updates', () => {
    it('should immediately update optimistic state', () => {
      const initialState = tasksReducer(
        undefined,
        setTasks([mockTask]),
      );

      const requestId = 'req_123';
      const newState = tasksReducer(
        initialState,
        updateTaskLaneOptimistic({
          taskId: 'task-1',
          fromLane: 'backlog',
          toLane: 'in-progress',
          requestId,
        }),
      );

      // Verify optimistic state updated
      expect(newState.optimisticTasks.backlog).toHaveLength(0);
      expect(newState.optimisticTasks['in-progress']).toHaveLength(1);
      expect(newState.optimisticTasks['in-progress'][0].lane).toBe('in-progress');

      // Verify server truth unchanged
      expect(newState.tasks.backlog).toHaveLength(1);
      expect(newState.tasks['in-progress']).toHaveLength(0);

      // Verify pending request tracked
      expect(newState.pending[requestId]).toBeDefined();
      expect(newState.pending[requestId].taskId).toBe('task-1');
    });

    it('should commit optimistic update on server success', () => {
      const initialState = tasksReducer(
        undefined,
        setTasks([mockTask]),
      );

      const requestId = 'req_123';
      const afterOptimistic = tasksReducer(
        initialState,
        updateTaskLaneOptimistic({
          taskId: 'task-1',
          fromLane: 'backlog',
          toLane: 'in-progress',
          requestId,
        }),
      );

      // Server confirms with version 4
      const updatedTask: Task = {
        ...mockTask,
        lane: 'in-progress',
        version: 4,
        updatedAt: '2024-02-14T11:00:00Z',
      };

      const afterCommit = tasksReducer(
        afterOptimistic,
        commitTaskLaneUpdate({
          requestId,
          updatedTask,
        }),
      );

      // Verify server truth updated
      expect(afterCommit.tasks.backlog).toHaveLength(0);
      expect(afterCommit.tasks['in-progress']).toHaveLength(1);
      expect(afterCommit.tasks['in-progress'][0].version).toBe(4);

      // Verify optimistic state synced
      expect(afterCommit.optimisticTasks['in-progress']).toHaveLength(1);
      expect(afterCommit.optimisticTasks['in-progress'][0].version).toBe(4);

      // Verify pending request removed
      expect(afterCommit.pending[requestId]).toBeUndefined();
    });

    it('should rollback optimistic update on server failure', () => {
      const initialState = tasksReducer(
        undefined,
        setTasks([mockTask]),
      );

      const requestId = 'req_123';
      const afterOptimistic = tasksReducer(
        initialState,
        updateTaskLaneOptimistic({
          taskId: 'task-1',
          fromLane: 'backlog',
          toLane: 'in-progress',
          requestId,
        }),
      );

      // Verify optimistic state changed
      expect(afterOptimistic.optimisticTasks['in-progress']).toHaveLength(1);

      // Rollback
      const afterRollback = tasksReducer(
        afterOptimistic,
        rollbackTaskLaneUpdate({ requestId }),
      );

      // Verify optimistic state restored from server truth
      expect(afterRollback.optimisticTasks.backlog).toHaveLength(1);
      expect(afterRollback.optimisticTasks['in-progress']).toHaveLength(0);

      // Verify pending request removed
      expect(afterRollback.pending[requestId]).toBeUndefined();
    });
  });
});
