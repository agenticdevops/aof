/**
 * TaskComment component - displays a single task comment.
 */

import React from 'react';
import ReactMarkdown from 'react-markdown';
import type { TaskComment as TaskCommentType } from '../types/comments';

/**
 * Props for TaskComment component.
 */
interface TaskCommentProps {
  /** Comment object */
  comment: TaskCommentType;

  /** Whether markdown rendering is enabled */
  enableMarkdown?: boolean;

  /** Whether this is the current user's comment */
  isOwnComment?: boolean;

  /** Callback when edit button is clicked */
  onEdit?: () => void;

  /** Callback when delete button is clicked */
  onDelete?: () => void;
}

/**
 * Format relative time from timestamp.
 */
function formatRelativeTime(timestamp: string): string {
  const now = new Date().getTime();
  const then = new Date(timestamp).getTime();
  const diffMs = now - then;
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  if (diffSec < 60) return 'Just now';
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHour < 24) return `${diffHour}h ago`;
  return `${diffDay}d ago`;
}

/**
 * TaskComment component.
 */
export function TaskComment({
  comment,
  enableMarkdown = true,
  isOwnComment = false,
  onEdit,
  onDelete,
}: TaskCommentProps): React.ReactElement {
  const relativeTime = formatRelativeTime(comment.timestamp);

  return (
    <div className="flex gap-3 py-3 border-b border-gray-200 dark:border-gray-700 last:border-b-0">
      {/* Avatar */}
      <div className="flex-shrink-0">
        {comment.authorAvatar ? (
          <div className="w-8 h-8 rounded-full bg-gray-200 dark:bg-gray-700 flex items-center justify-center text-lg">
            {comment.authorAvatar}
          </div>
        ) : (
          <div className="w-8 h-8 rounded-full bg-gray-300 dark:bg-gray-600 flex items-center justify-center text-xs font-medium text-gray-700 dark:text-gray-300">
            {comment.authorName.substring(0, 2).toUpperCase()}
          </div>
        )}
      </div>

      {/* Comment Content */}
      <div className="flex-1 min-w-0">
        {/* Header: Author name + timestamp */}
        <div className="flex items-center justify-between gap-2 mb-1">
          <div className="flex items-baseline gap-2">
            <span className="text-sm font-semibold text-gray-900 dark:text-white">
              {comment.authorName}
            </span>
            <time className="text-xs text-gray-500 dark:text-gray-400" dateTime={comment.timestamp}>
              {relativeTime}
            </time>
          </div>

          {/* Action buttons (if own comment) */}
          {isOwnComment && (onEdit || onDelete) && (
            <div className="flex gap-2">
              {onEdit && (
                <button
                  onClick={onEdit}
                  className="text-xs text-blue-600 dark:text-blue-400 hover:underline focus:outline-none focus:ring-2 focus:ring-blue-500 rounded"
                  aria-label="Edit comment"
                >
                  Edit
                </button>
              )}
              {onDelete && (
                <button
                  onClick={onDelete}
                  className="text-xs text-red-600 dark:text-red-400 hover:underline focus:outline-none focus:ring-2 focus:ring-red-500 rounded"
                  aria-label="Delete comment"
                >
                  Delete
                </button>
              )}
            </div>
          )}
        </div>

        {/* Comment text */}
        <div className="text-sm text-gray-700 dark:text-gray-300 break-words">
          {enableMarkdown ? (
            <ReactMarkdown
              components={{
                a: ({ node, ...props }) => (
                  <a
                    {...props}
                    className="text-blue-600 dark:text-blue-400 hover:underline"
                    target="_blank"
                    rel="noopener noreferrer"
                  />
                ),
                code: ({ node, inline, ...props }) =>
                  inline ? (
                    <code
                      {...props}
                      className="bg-gray-100 dark:bg-gray-800 px-1 py-0.5 rounded text-xs font-mono"
                    />
                  ) : (
                    <code
                      {...props}
                      className="block bg-gray-100 dark:bg-gray-800 p-2 rounded text-xs font-mono overflow-x-auto my-2"
                    />
                  ),
                p: ({ node, ...props }) => <p {...props} className="mb-1" />,
              }}
            >
              {comment.text}
            </ReactMarkdown>
          ) : (
            <p>{comment.text}</p>
          )}
        </div>
      </div>
    </div>
  );
}
