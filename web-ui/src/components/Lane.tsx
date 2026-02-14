/**
 * Lane component - droppable container for Kanban tasks.
 * Represents a single column in the Kanban board (backlog, assigned, in-progress, review, done).
 */

import React from 'react';
import { useDroppable } from '@dnd-kit/core';
import { SortableContext, verticalListSortingStrategy } from '@dnd-kit/sortable';
import type { Task, TaskLane } from '../types/tasks';
import { TaskCard } from './TaskCard';

/**
 * Component props.
 */
export interface LaneProps {
  /** Lane identifier */
  laneId: TaskLane;

  /** Lane display name */
  laneName: string;

  /** Tasks in this lane */
  tasks: Task[];

  /** Optional className for styling */
  className?: string;
}

/**
 * Get lane header background color.
 */
function getLaneHeaderColor(laneId: TaskLane): string {
  switch (laneId) {
    case 'backlog':
      return 'bg-slate-100 dark:bg-slate-800';
    case 'assigned':
      return 'bg-blue-100 dark:bg-blue-900';
    case 'in-progress':
      return 'bg-orange-100 dark:bg-orange-900';
    case 'review':
      return 'bg-yellow-100 dark:bg-yellow-900';
    case 'done':
      return 'bg-green-100 dark:bg-green-900';
    default:
      return 'bg-gray-100 dark:bg-gray-800';
  }
}

/**
 * Get lane header text color.
 */
function getLaneHeaderTextColor(laneId: TaskLane): string {
  switch (laneId) {
    case 'backlog':
      return 'text-slate-700 dark:text-slate-200';
    case 'assigned':
      return 'text-blue-700 dark:text-blue-200';
    case 'in-progress':
      return 'text-orange-700 dark:text-orange-200';
    case 'review':
      return 'text-yellow-700 dark:text-yellow-200';
    case 'done':
      return 'text-green-700 dark:text-green-200';
    default:
      return 'text-gray-700 dark:text-gray-200';
  }
}

/**
 * Empty state component for lane.
 */
function EmptyLaneState({ laneName }: { laneName: string }): React.ReactElement {
  return (
    <div className="flex flex-col items-center justify-center py-8 text-center">
      <div className="text-4xl mb-2 opacity-50">📋</div>
      <p className="text-sm text-gray-500 dark:text-gray-400">No tasks in {laneName}</p>
    </div>
  );
}

/**
 * Lane component.
 *
 * Features:
 * - Droppable zone with dnd-kit useDroppable hook
 * - SortableContext for task ordering within lane
 * - Color-coded header by lane type
 * - Task count badge in header
 * - Empty state when no tasks
 * - Fixed width and min-height for consistent layout
 * - Scrollable content area
 *
 * @example
 * ```tsx
 * <Lane
 *   laneId="in-progress"
 *   laneName="In Progress"
 *   tasks={[task1, task2, task3]}
 * />
 * ```
 */
export function Lane({
  laneId,
  laneName,
  tasks,
  className = '',
}: LaneProps): React.ReactElement {
  const { setNodeRef, isOver } = useDroppable({
    id: laneId,
  });

  const headerBgColor = getLaneHeaderColor(laneId);
  const headerTextColor = getLaneHeaderTextColor(laneId);
  const taskIds = tasks.map((task) => task.id);

  return (
    <div
      ref={setNodeRef}
      className={`flex flex-col bg-gray-50 dark:bg-gray-900 rounded-lg border-2 ${isOver ? 'border-blue-400 border-dashed bg-blue-50 dark:bg-blue-950' : 'border-gray-200 dark:border-gray-700'} transition-colors ${className}`}
      style={{ minHeight: '500px', width: '280px' }}
    >
      {/* Lane header */}
      <div className={`p-3 rounded-t-lg ${headerBgColor}`}>
        <div className="flex items-center justify-between">
          <h3 className={`font-semibold text-sm ${headerTextColor}`}>{laneName}</h3>
          {/* Task count badge */}
          <span
            className={`px-2 py-0.5 text-xs font-medium rounded-full ${headerBgColor} ${headerTextColor} bg-opacity-50`}
            aria-label={`${tasks.length} tasks in ${laneName}`}
          >
            {tasks.length}
          </span>
        </div>
      </div>

      {/* Tasks container */}
      <div className="flex-1 p-2 overflow-y-auto">
        <SortableContext items={taskIds} strategy={verticalListSortingStrategy}>
          {tasks.length === 0 ? (
            <EmptyLaneState laneName={laneName} />
          ) : (
            tasks.map((task) => <TaskCard key={task.id} task={task} />)
          )}
        </SortableContext>
      </div>
    </div>
  );
}
