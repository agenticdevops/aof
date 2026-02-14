/**
 * TaskDetail modal component - displays full task context with tabs.
 */

import React, { useState } from 'react';
import { useSelector } from 'react-redux';
import { Modal } from './Modal';
import type { Task } from '../types/tasks';
import type { RootState } from '../store';

/**
 * Props for TaskDetail component.
 */
interface TaskDetailProps {
  /** Task ID to display */
  taskId: string | null;

  /** Callback when modal closes */
  onClose: () => void;
}

/**
 * Tab type for task detail modal.
 */
type TabType = 'overview' | 'comments' | 'history';

/**
 * Get task by ID from Redux store.
 */
function useTaskById(taskId: string | null): Task | undefined {
  return useSelector((state: RootState) => {
    if (!taskId) return undefined;

    // Search all lanes for task
    for (const lane of Object.values(state.tasks.optimisticTasks)) {
      const task = lane.find((t) => t.id === taskId);
      if (task) return task;
    }
    return undefined;
  });
}

/**
 * Get status badge color.
 */
function getStatusColor(lane: string): string {
  const colorMap: Record<string, string> = {
    backlog: 'bg-gray-500',
    assigned: 'bg-blue-500',
    'in-progress': 'bg-orange-500',
    review: 'bg-yellow-500',
    done: 'bg-green-500',
  };
  return colorMap[lane] || 'bg-gray-500';
}

/**
 * TaskDetail component.
 */
export function TaskDetail({ taskId, onClose }: TaskDetailProps): React.ReactElement {
  const task = useTaskById(taskId);
  const [activeTab, setActiveTab] = useState<TabType>('overview');

  const isOpen = taskId !== null;

  if (!isOpen || !task) {
    return <></>;
  }

  const statusColor = getStatusColor(task.lane);

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={task.title}>
      {/* Tabs */}
      <div className="border-b border-gray-200 dark:border-gray-700 mb-4">
        <nav className="flex gap-4" aria-label="Task detail tabs">
          <button
            onClick={() => setActiveTab('overview')}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
              activeTab === 'overview'
                ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                : 'border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'
            }`}
            aria-selected={activeTab === 'overview'}
            role="tab"
          >
            Overview
          </button>
          <button
            onClick={() => setActiveTab('comments')}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
              activeTab === 'comments'
                ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                : 'border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'
            }`}
            aria-selected={activeTab === 'comments'}
            role="tab"
          >
            Comments
          </button>
          <button
            onClick={() => setActiveTab('history')}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
              activeTab === 'history'
                ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                : 'border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'
            }`}
            aria-selected={activeTab === 'history'}
            role="tab"
          >
            History
          </button>
        </nav>
      </div>

      {/* Tab Content */}
      <div role="tabpanel">
        {activeTab === 'overview' && (
          <div className="space-y-4">
            {/* Status Badge */}
            <div>
              <span
                className={`inline-flex items-center px-3 py-1 rounded-full text-xs font-medium text-white ${statusColor}`}
              >
                {task.lane.replace('-', ' ').toUpperCase()}
              </span>
            </div>

            {/* Description */}
            <div>
              <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Description
              </h3>
              <p className="text-sm text-gray-900 dark:text-white">
                {task.description || 'No description provided.'}
              </p>
            </div>

            {/* Assignee */}
            {task.assignee && (
              <div>
                <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Assigned To
                </h3>
                <div className="flex items-center gap-2">
                  <div className="w-8 h-8 rounded-full bg-gray-300 dark:bg-gray-600 flex items-center justify-center text-xs font-medium">
                    {task.assignee.substring(0, 2).toUpperCase()}
                  </div>
                  <span className="text-sm text-gray-900 dark:text-white">{task.assignee}</span>
                </div>
              </div>
            )}

            {/* Priority */}
            <div>
              <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Priority
              </h3>
              <span className="text-sm text-gray-900 dark:text-white capitalize">
                {task.priority}
              </span>
            </div>

            {/* Tags */}
            {task.tags && task.tags.length > 0 && (
              <div>
                <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Tags
                </h3>
                <div className="flex flex-wrap gap-2">
                  {task.tags.map((tag, index) => (
                    <span
                      key={index}
                      className="px-2 py-1 bg-gray-100 dark:bg-gray-700 text-xs rounded"
                    >
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
            )}

            {/* Metadata */}
            <div className="grid grid-cols-2 gap-4 pt-4 border-t border-gray-200 dark:border-gray-700">
              <div>
                <h3 className="text-xs font-medium text-gray-500 dark:text-gray-400">
                  Version
                </h3>
                <p className="text-sm text-gray-900 dark:text-white mt-1">{task.version}</p>
              </div>
              <div>
                <h3 className="text-xs font-medium text-gray-500 dark:text-gray-400">
                  Task ID
                </h3>
                <p className="text-xs text-gray-900 dark:text-white mt-1 font-mono">{task.id}</p>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'comments' && (
          <div className="text-sm text-gray-500 dark:text-gray-400">
            Comments feature coming in Task 04-03-07
          </div>
        )}

        {activeTab === 'history' && (
          <div className="text-sm text-gray-500 dark:text-gray-400">
            History timeline coming in Task 04-03-06
          </div>
        )}
      </div>
    </Modal>
  );
}
