/**
 * KanbanBoard component - main Kanban board with drag-and-drop.
 * Manages 5 lanes: backlog, assigned, in-progress, review, done.
 */

import React, { useEffect, useState } from 'react';
import { DndContext, type DragEndEvent } from '@dnd-kit/core';
import { useTaskManagement } from '../hooks/useTaskManagement';
import { useDndSensors, hasValidDestination, getTaskIdFromEvent, getDestinationLaneFromEvent } from '../utils/dndConfig';
import { Lane } from './Lane';
import type { TaskLane } from '../types/tasks';

/**
 * Component props.
 */
export interface KanbanBoardProps {
  /** Optional className for styling */
  className?: string;
}

/**
 * Lane configuration.
 */
interface LaneConfig {
  id: TaskLane;
  name: string;
}

/**
 * All 5 lanes in order.
 */
const LANES: LaneConfig[] = [
  { id: 'backlog', name: 'Backlog' },
  { id: 'assigned', name: 'Assigned' },
  { id: 'in-progress', name: 'In Progress' },
  { id: 'review', name: 'Review' },
  { id: 'done', name: 'Done' },
];

/**
 * Toast notification component.
 */
function Toast({
  message,
  type = 'info',
  onClose,
}: {
  message: string;
  type?: 'info' | 'success' | 'error';
  onClose: () => void;
}): React.ReactElement {
  useEffect(() => {
    const timer = setTimeout(onClose, 3000);
    return () => clearTimeout(timer);
  }, [onClose]);

  const bgColor = type === 'error' ? 'bg-red-600' : type === 'success' ? 'bg-green-600' : 'bg-blue-600';

  return (
    <div className={`fixed top-4 right-4 z-50 ${bgColor} text-white px-4 py-3 rounded-lg shadow-lg flex items-center gap-2 animate-slide-in`}>
      <span>{type === 'error' ? '❌' : type === 'success' ? '✅' : 'ℹ️'}</span>
      <span>{message}</span>
    </div>
  );
}

/**
 * Loading skeleton for lane.
 */
function LaneSkeleton({ name }: { name: string }): React.ReactElement {
  return (
    <div className="flex flex-col bg-gray-50 dark:bg-gray-900 rounded-lg border-2 border-gray-200 dark:border-gray-700" style={{ minHeight: '500px', width: '280px' }}>
      <div className="p-3 rounded-t-lg bg-gray-100 dark:bg-gray-800">
        <div className="flex items-center justify-between">
          <h3 className="font-semibold text-sm text-gray-700 dark:text-gray-300">{name}</h3>
          <span className="px-2 py-0.5 text-xs font-medium rounded-full bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-400">
            ...
          </span>
        </div>
      </div>
      <div className="flex-1 p-2 space-y-2">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-3 animate-pulse">
            <div className="h-4 bg-gray-300 dark:bg-gray-600 rounded w-3/4 mb-2" />
            <div className="h-3 bg-gray-300 dark:bg-gray-600 rounded w-full mb-1" />
            <div className="h-3 bg-gray-300 dark:bg-gray-600 rounded w-5/6" />
          </div>
        ))}
      </div>
    </div>
  );
}

/**
 * KanbanBoard component.
 *
 * Features:
 * - 5 lanes: Backlog, Assigned, In-Progress, Review, Done
 * - Drag-and-drop between lanes with dnd-kit
 * - Optimistic updates (instant visual feedback)
 * - Server sync with POST /api/tasks/move
 * - Conflict resolution (409 Conflict → rollback)
 * - Error handling with retry logic (5xx errors)
 * - Loading state (skeleton lanes)
 * - Toast notifications (success/error/info)
 * - Horizontal scroll on mobile
 * - Responsive layout
 *
 * @example
 * ```tsx
 * <KanbanBoard />
 * ```
 */
export function KanbanBoard({ className = '' }: KanbanBoardProps): React.ReactElement {
  const { tasks, loading, error, moveTask, refetchTasks } = useTaskManagement();
  const sensors = useDndSensors();
  const [toast, setToast] = useState<{ message: string; type: 'info' | 'success' | 'error' } | null>(null);

  /**
   * Fetch tasks on mount.
   */
  useEffect(() => {
    refetchTasks();
  }, [refetchTasks]);

  /**
   * Handle drag end event.
   */
  const handleDragEnd = async (event: DragEndEvent) => {
    // Check if drag has valid destination
    if (!hasValidDestination(event)) {
      return;
    }

    const taskId = getTaskIdFromEvent(event);
    const newLaneId = getDestinationLaneFromEvent(event);

    if (!newLaneId) {
      return;
    }

    // Find current lane
    let currentLane: TaskLane | undefined;
    for (const lane of Object.keys(tasks) as TaskLane[]) {
      if (tasks[lane].some((t) => t.id === taskId)) {
        currentLane = lane;
        break;
      }
    }

    // If task is already in destination lane, do nothing
    if (currentLane === newLaneId) {
      return;
    }

    try {
      // moveTask handles optimistic update and server sync
      await moveTask(taskId, newLaneId as TaskLane);
      setToast({ message: `Task moved to ${newLaneId}`, type: 'success' });
    } catch (err) {
      console.error('Task move failed:', err);
      setToast({ message: 'Failed to move task', type: 'error' });
    }
  };

  /**
   * Show error from useTaskManagement.
   */
  useEffect(() => {
    if (error) {
      setToast({ message: error, type: 'error' });
    }
  }, [error]);

  // Loading state
  if (loading && Object.keys(tasks).length === 0) {
    return (
      <div className={`flex gap-4 overflow-x-auto pb-4 ${className}`}>
        {LANES.map((lane) => (
          <LaneSkeleton key={lane.id} name={lane.name} />
        ))}
      </div>
    );
  }

  return (
    <div className={className}>
      <DndContext sensors={sensors} onDragEnd={handleDragEnd}>
        <div className="flex gap-4 overflow-x-auto pb-4">
          {LANES.map((lane) => (
            <Lane
              key={lane.id}
              laneId={lane.id}
              laneName={lane.name}
              tasks={tasks[lane.id] || []}
            />
          ))}
        </div>
      </DndContext>

      {/* Toast notifications */}
      {toast && (
        <Toast
          message={toast.message}
          type={toast.type}
          onClose={() => setToast(null)}
        />
      )}
    </div>
  );
}
