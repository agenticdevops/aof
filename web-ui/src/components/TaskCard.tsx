/**
 * TaskCard component - draggable task card for Kanban board.
 * Integrates with dnd-kit for drag-and-drop functionality.
 */

import React from 'react';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import type { Task, TaskStatus } from '../types/tasks';

/**
 * Component props.
 */
export interface TaskCardProps {
  /** Task data */
  task: Task;

  /** Optional className for styling */
  className?: string;
}

/**
 * Get border color based on task status.
 */
function getStatusBorderColor(status: TaskStatus): string {
  switch (status) {
    case 'completed':
      return 'border-l-green-500';
    case 'active':
      return 'border-l-orange-500';
    case 'blocked':
      return 'border-l-red-500';
    case 'cancelled':
      return 'border-l-gray-400';
    case 'pending':
    default:
      return 'border-l-gray-300';
  }
}

/**
 * Get status badge color.
 */
function getStatusBadgeColor(status: TaskStatus): string {
  switch (status) {
    case 'completed':
      return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
    case 'active':
      return 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200';
    case 'blocked':
      return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
    case 'cancelled':
      return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300';
    case 'pending':
    default:
      return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
  }
}

/**
 * Get priority badge color.
 */
function getPriorityBadgeColor(priority: Task['priority']): string {
  switch (priority) {
    case 'critical':
      return 'bg-red-600 text-white';
    case 'high':
      return 'bg-orange-500 text-white';
    case 'medium':
      return 'bg-yellow-500 text-white';
    case 'low':
      return 'bg-green-500 text-white';
    default:
      return 'bg-gray-400 text-white';
  }
}

/**
 * TaskCard component.
 *
 * Features:
 * - Draggable with dnd-kit useSortable hook
 * - Visual feedback during drag (opacity, shadow)
 * - Status-based border color
 * - Displays title, description (truncated), assignee, status, version
 * - Keyboard accessible (role, tabIndex, aria-label)
 *
 * @example
 * ```tsx
 * <TaskCard
 *   task={{
 *     id: 'task-1',
 *     title: 'Fix login bug',
 *     description: 'Users cannot log in with SSO',
 *     lane: 'in-progress',
 *     assignedTo: 'agent-1',
 *     version: 3,
 *     status: 'active',
 *     createdAt: '2024-02-14T10:00:00Z',
 *     updatedAt: '2024-02-14T12:00:00Z',
 *   }}
 * />
 * ```
 */
export function TaskCard({ task, className = '' }: TaskCardProps): React.ReactElement {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: task.id,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  const borderColor = getStatusBorderColor(task.status);
  const statusBadgeColor = getStatusBadgeColor(task.status);
  const priorityBadgeColor = task.priority ? getPriorityBadgeColor(task.priority) : null;

  const descriptionId = `task-${task.id}-description`;
  const statusId = `task-${task.id}-status`;

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`bg-white dark:bg-gray-800 rounded-lg shadow-sm border-l-4 ${borderColor} border border-gray-200 dark:border-gray-700 p-3 mb-2 cursor-grab active:cursor-grabbing ${isDragging ? 'shadow-2xl' : ''} ${className}`}
      {...attributes}
      {...listeners}
      aria-label={`Task: ${task.title}, in ${task.lane} lane`}
      aria-describedby={`${descriptionId} ${statusId}`}
    >
      {/* Drag handle indicator */}
      <div className="flex items-start gap-2 mb-2">
        <div className="flex-shrink-0 text-gray-400 dark:text-gray-500 mt-1">
          <svg
            className="w-4 h-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M4 8h16M4 16h16"
            />
          </svg>
        </div>

        {/* Title */}
        <h4 className="flex-1 text-sm font-semibold text-gray-900 dark:text-gray-100 line-clamp-1">
          {task.title}
        </h4>

        {/* Priority badge */}
        {priorityBadgeColor && (
          <span className={`px-2 py-0.5 text-xs font-medium rounded ${priorityBadgeColor}`}>
            {task.priority}
          </span>
        )}
      </div>

      {/* Description */}
      <p id={descriptionId} className="text-xs text-gray-600 dark:text-gray-400 mb-2 line-clamp-2">
        {task.description}
      </p>

      {/* Tags */}
      {task.tags && task.tags.length > 0 && (
        <div className="flex flex-wrap gap-1 mb-2">
          {task.tags.slice(0, 2).map((tag) => (
            <span
              key={tag}
              className="px-1.5 py-0.5 text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded"
            >
              {tag}
            </span>
          ))}
          {task.tags.length > 2 && (
            <span className="px-1.5 py-0.5 text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded">
              +{task.tags.length - 2}
            </span>
          )}
        </div>
      )}

      {/* Footer: Assignee, Status, Version */}
      <div className="flex items-center justify-between pt-2 border-t border-gray-100 dark:border-gray-700">
        {/* Assignee */}
        <div className="flex items-center gap-1 text-xs text-gray-500 dark:text-gray-400">
          {task.assignedTo ? (
            <>
              <span>👤</span>
              <span className="truncate max-w-[80px]">{task.assignedTo}</span>
            </>
          ) : (
            <span className="text-gray-400 dark:text-gray-500">Unassigned</span>
          )}
        </div>

        {/* Status badge */}
        <span
          id={statusId}
          className={`px-2 py-0.5 text-xs font-medium rounded ${statusBadgeColor}`}
          aria-label={`Status: ${task.status}`}
        >
          {task.status}
        </span>

        {/* Version */}
        <span className="text-xs text-gray-400 dark:text-gray-500">v{task.version}</span>
      </div>
    </div>
  );
}
