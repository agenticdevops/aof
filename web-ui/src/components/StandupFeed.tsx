/**
 * StandupFeed component - displays standup results in chronological order.
 * Shows expandable agent responses with DID/DOING/BLOCKERS sections.
 */

import React, { useState } from 'react';
import type { StandupResult, StandupResponseRecord } from '../types/coordination';

/**
 * Component props.
 */
export interface StandupFeedProps {
  /** Latest standup result */
  standupResult: StandupResult | null;

  /** Trigger standup function */
  onTriggerStandup: () => void;

  /** Loading state */
  isLoading?: boolean;

  /** Optional className for styling */
  className?: string;
}

/**
 * Format ISO 8601 timestamp for display.
 */
function formatStandupDate(timestamp: string): string {
  try {
    const date = new Date(timestamp);
    return date.toLocaleString('en-US', {
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
    });
  } catch {
    return timestamp;
  }
}

/**
 * Format timestamp for agent response card.
 */
function formatResponseTime(timestamp: string): string {
  try {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', {
      hour: 'numeric',
      minute: '2-digit',
    });
  } catch {
    return timestamp;
  }
}

/**
 * Agent response card component.
 */
function AgentResponseCard({
  response,
}: {
  response: StandupResponseRecord;
}): React.ReactElement {
  const [expanded, setExpanded] = useState(true);

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 shadow-sm">
      {/* Header: Agent name + expand button + token count */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 bg-gray-100 dark:bg-gray-700 rounded-full flex items-center justify-center text-lg">
            🤖
          </div>
          <h4 className="font-semibold text-gray-900 dark:text-gray-100">
            {response.agent_id}
          </h4>
        </div>

        <div className="flex items-center gap-3">
          {/* Token count badge */}
          <span className="text-xs bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 px-2 py-1 rounded">
            {response.token_count} tokens
          </span>

          {/* Timestamp */}
          <span className="text-xs text-gray-500 dark:text-gray-400">
            {formatResponseTime(response.timestamp)}
          </span>

          {/* Expand/collapse button */}
          <button
            onClick={() => setExpanded(!expanded)}
            className="text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 transition-colors"
            aria-label={expanded ? 'Collapse' : 'Expand'}
          >
            <svg
              className={`w-5 h-5 transition-transform ${expanded ? 'rotate-180' : ''}`}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M19 9l-7 7-7-7"
              />
            </svg>
          </button>
        </div>
      </div>

      {/* Expandable content */}
      {expanded && (
        <div className="space-y-3 mt-3 pt-3 border-t border-gray-100 dark:border-gray-700">
          {/* DID section (green) */}
          <div>
            <h5 className="text-xs font-semibold text-green-600 dark:text-green-400 uppercase mb-1">
              Did
            </h5>
            <p className="text-sm text-gray-700 dark:text-gray-300">
              {response.what_i_did || 'No updates'}
            </p>
          </div>

          {/* DOING section (blue) */}
          <div>
            <h5 className="text-xs font-semibold text-blue-600 dark:text-blue-400 uppercase mb-1">
              Doing
            </h5>
            <p className="text-sm text-gray-700 dark:text-gray-300">
              {response.what_im_doing || 'Nothing scheduled'}
            </p>
          </div>

          {/* BLOCKERS section (red or gray) */}
          <div>
            <h5 className="text-xs font-semibold text-red-600 dark:text-red-400 uppercase mb-1">
              Blockers
            </h5>
            {response.blockers.length > 0 ? (
              <ul className="list-disc list-inside text-sm text-red-700 dark:text-red-300 space-y-1">
                {response.blockers.map((blocker, idx) => (
                  <li key={idx}>{blocker}</li>
                ))}
              </ul>
            ) : (
              <p className="text-sm text-gray-500 dark:text-gray-400 italic">
                No blockers
              </p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

/**
 * StandupFeed component.
 *
 * Displays standup results with:
 * - Date header
 * - Optional AI summary
 * - Expandable agent response cards
 * - Trigger standup button
 *
 * @example
 * ```tsx
 * <StandupFeed
 *   standupResult={latestStandup}
 *   onTriggerStandup={triggerStandup}
 *   isLoading={loading}
 * />
 * ```
 */
export function StandupFeed({
  standupResult,
  onTriggerStandup,
  isLoading = false,
  className = '',
}: StandupFeedProps): React.ReactElement {
  // Empty state: no standup results
  if (!standupResult || standupResult.responses.length === 0) {
    return (
      <div className={`bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-8 text-center ${className}`}>
        <div className="text-gray-400 dark:text-gray-500 mb-4">
          <svg
            className="w-16 h-16 mx-auto mb-4 opacity-50"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
            />
          </svg>
          <p className="text-lg font-medium mb-2">No standup results yet</p>
          <p className="text-sm mb-4">
            Trigger a standup to see what your agents are working on.
          </p>
        </div>

        <button
          onClick={onTriggerStandup}
          disabled={isLoading}
          className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded-lg font-medium transition-colors shadow-sm disabled:cursor-not-allowed"
        >
          {isLoading ? 'Triggering...' : 'Trigger Standup Now'}
        </button>
      </div>
    );
  }

  const formattedDate = formatStandupDate(standupResult.triggered_at);

  return (
    <div className={className}>
      {/* Header with date and trigger button */}
      <div className="bg-gradient-to-r from-purple-50 to-pink-50 dark:from-gray-800 dark:to-gray-750 rounded-lg p-4 mb-4 shadow-sm">
        <div className="flex items-center justify-between mb-2">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Daily Standup
          </h2>

          <button
            onClick={onTriggerStandup}
            disabled={isLoading}
            className="px-3 py-1.5 bg-purple-600 hover:bg-purple-700 disabled:bg-gray-400 text-white text-sm rounded-lg font-medium transition-colors shadow-sm disabled:cursor-not-allowed"
          >
            {isLoading ? 'Triggering...' : 'Trigger Standup'}
          </button>
        </div>

        <p className="text-sm text-gray-600 dark:text-gray-400">
          {formattedDate}
        </p>

        {/* AI-generated summary (if present) */}
        {standupResult.summary && (
          <div className="mt-3 pt-3 border-t border-purple-100 dark:border-gray-700">
            <h3 className="text-sm font-semibold text-purple-700 dark:text-purple-300 mb-1">
              Summary
            </h3>
            <p className="text-sm text-gray-700 dark:text-gray-300">
              {standupResult.summary}
            </p>
          </div>
        )}
      </div>

      {/* Agent responses */}
      <div className="space-y-3">
        <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
          Agent Responses ({standupResult.responses.length})
        </h3>

        {standupResult.responses.map((response) => (
          <AgentResponseCard key={response.agent_id} response={response} />
        ))}
      </div>
    </div>
  );
}
