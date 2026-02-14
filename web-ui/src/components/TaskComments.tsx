/**
 * TaskComments component - displays comments list and add comment input.
 */

import React, { useState, useEffect } from 'react';
import { TaskComment } from './TaskComment';
import { generateOptimisticCommentId, type TaskComment as TaskCommentType } from '../types/comments';

/**
 * Props for TaskComments component.
 */
interface TaskCommentsProps {
  /** Task ID to show comments for */
  taskId: string;
}

/**
 * TaskComments component.
 */
export function TaskComments({ taskId }: TaskCommentsProps): React.ReactElement {
  const [comments, setComments] = useState<TaskCommentType[]>([]);
  const [loading, setLoading] = useState(true);
  const [inputValue, setInputValue] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Fetch comments for task.
   */
  useEffect(() => {
    const fetchComments = async () => {
      try {
        setLoading(true);
        const response = await fetch(`/api/tasks/${taskId}/comments`);
        if (!response.ok) {
          if (response.status === 404) {
            // No comments endpoint yet, use empty array
            setComments([]);
            setLoading(false);
            return;
          }
          throw new Error(`Failed to fetch comments: ${response.statusText}`);
        }
        const data: TaskCommentType[] = await response.json();
        setComments(data);
      } catch (err) {
        console.error('Error fetching comments:', err);
        setError('Failed to load comments');
        setComments([]);
      } finally {
        setLoading(false);
      }
    };

    fetchComments();
  }, [taskId]);

  /**
   * Handle submit comment.
   */
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!inputValue.trim() || submitting) return;

    const optimisticComment: TaskCommentType = {
      id: generateOptimisticCommentId(),
      taskId,
      authorId: 'user_1', // TODO: Get actual user ID
      authorName: 'You', // TODO: Get actual user name
      authorAvatar: '👤',
      text: inputValue,
      timestamp: new Date().toISOString(),
    };

    // Optimistic update
    setComments((prev) => [...prev, optimisticComment]);
    setInputValue('');
    setSubmitting(true);

    try {
      const response = await fetch(`/api/tasks/${taskId}/comments`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          text: inputValue,
          version: 1, // TODO: Get task version
        }),
      });

      if (!response.ok) {
        throw new Error(`Failed to post comment: ${response.statusText}`);
      }

      const confirmedComment: TaskCommentType = await response.json();

      // Replace optimistic comment with server-confirmed comment
      setComments((prev) =>
        prev.map((c) => (c.id === optimisticComment.id ? confirmedComment : c))
      );
    } catch (err) {
      console.error('Error posting comment:', err);
      setError('Failed to post comment');

      // Rollback optimistic update
      setComments((prev) => prev.filter((c) => c.id !== optimisticComment.id));
      setInputValue(inputValue); // Restore input
    } finally {
      setSubmitting(false);
    }
  };

  /**
   * Handle edit comment (placeholder).
   */
  const handleEdit = (commentId: string) => {
    console.log('Edit comment:', commentId);
    // TODO: Implement edit functionality
  };

  /**
   * Handle delete comment (placeholder).
   */
  const handleDelete = (commentId: string) => {
    console.log('Delete comment:', commentId);
    // TODO: Implement delete functionality
  };

  return (
    <div className="space-y-4">
      {/* Comments List */}
      {loading ? (
        <div className="text-center py-8">
          <p className="text-sm text-gray-500 dark:text-gray-400">Loading comments...</p>
        </div>
      ) : comments.length === 0 ? (
        <div className="text-center py-8">
          <p className="text-sm text-gray-500 dark:text-gray-400">
            No comments yet. Be the first to comment!
          </p>
        </div>
      ) : (
        <div className="space-y-0">
          {comments.map((comment) => (
            <TaskComment
              key={comment.id}
              comment={comment}
              enableMarkdown={true}
              isOwnComment={comment.authorId === 'user_1'}
              onEdit={() => handleEdit(comment.id)}
              onDelete={() => handleDelete(comment.id)}
            />
          ))}
        </div>
      )}

      {/* Error message */}
      {error && (
        <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded">
          <p className="text-sm text-red-800 dark:text-red-200">{error}</p>
        </div>
      )}

      {/* Add Comment Input */}
      <form onSubmit={handleSubmit} className="border-t border-gray-200 dark:border-gray-700 pt-4">
        <label htmlFor="comment-input" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Add a comment
        </label>
        <textarea
          id="comment-input"
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          placeholder="Write a comment... (Markdown supported)"
          rows={3}
          disabled={submitting}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
        />
        <div className="flex justify-end mt-2">
          <button
            type="submit"
            disabled={!inputValue.trim() || submitting}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {submitting ? 'Posting...' : 'Post Comment'}
          </button>
        </div>
      </form>
    </div>
  );
}
