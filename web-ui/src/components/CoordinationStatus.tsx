/**
 * CoordinationStatus component - displays coordination mode and token overhead.
 * Shows token overhead gauge with threshold indicator and mode badge.
 */

import React, { useState } from 'react';
import type { CoordinationMetrics, CoordinationMode } from '../types/coordination';

/**
 * Component props.
 */
export interface CoordinationStatusProps {
  /** Coordination metrics */
  metrics: CoordinationMetrics | null;

  /** Force mode change function */
  onForceMode: (mode: CoordinationMode) => void;

  /** Loading state */
  isLoading?: boolean;

  /** Compact mode (shows only badge and overhead %) */
  compact?: boolean;

  /** Optional className for styling */
  className?: string;
}

/**
 * Get mode badge color class.
 */
function getModeBadgeColor(mode: CoordinationMode): string {
  switch (mode) {
    case 'Full':
      return 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300';
    case 'Standard':
      return 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300';
    case 'Reduced':
      return 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300';
    case 'HeartbeatOnly':
      return 'bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300';
    case 'Disabled':
      return 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300';
    default:
      return 'bg-gray-100 text-gray-700 dark:bg-gray-900 dark:text-gray-300';
  }
}

/**
 * Get overhead gauge color class based on percentage.
 */
function getOverheadColor(overhead: number): string {
  if (overhead < 20) {
    return 'bg-green-500 dark:bg-green-400';
  } else if (overhead < 30) {
    return 'bg-yellow-500 dark:bg-yellow-400';
  } else {
    return 'bg-red-500 dark:bg-red-400';
  }
}

/**
 * Get overhead text color class.
 */
function getOverheadTextColor(overhead: number): string {
  if (overhead < 20) {
    return 'text-green-600 dark:text-green-400';
  } else if (overhead < 30) {
    return 'text-yellow-600 dark:text-yellow-400';
  } else {
    return 'text-red-600 dark:text-red-400';
  }
}

/**
 * Format token count with K/M suffixes.
 */
function formatTokenCount(count: number): string {
  if (count >= 1_000_000) {
    return `${(count / 1_000_000).toFixed(1)}M`;
  } else if (count >= 1_000) {
    return `${(count / 1_000).toFixed(1)}K`;
  } else {
    return count.toString();
  }
}

/**
 * Mode selector dropdown component.
 */
function ModeSelector({
  currentMode,
  onSelect,
  isLoading,
}: {
  currentMode: CoordinationMode;
  onSelect: (mode: CoordinationMode) => void;
  isLoading: boolean;
}): React.ReactElement {
  const [isOpen, setIsOpen] = useState(false);

  const modes: CoordinationMode[] = [
    'Full',
    'Standard',
    'Reduced',
    'HeartbeatOnly',
    'Disabled',
  ];

  const handleSelect = (mode: CoordinationMode) => {
    onSelect(mode);
    setIsOpen(false);
  };

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        disabled={isLoading}
        className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${getModeBadgeColor(
          currentMode
        )} hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50`}
      >
        {currentMode}
        <svg
          className={`w-4 h-4 inline ml-1 transition-transform ${isOpen ? 'rotate-180' : ''}`}
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

      {isOpen && (
        <div className="absolute top-full mt-1 left-0 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg z-10 min-w-[150px]">
          {modes.map((mode) => (
            <button
              key={mode}
              onClick={() => handleSelect(mode)}
              className={`block w-full text-left px-3 py-2 text-sm hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors ${
                mode === currentMode ? 'font-semibold' : ''
              }`}
            >
              {mode}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * CoordinationStatus component.
 *
 * Displays:
 * - Current coordination mode badge (clickable for mode override)
 * - Token overhead gauge with threshold indicator
 * - Token breakdown (heartbeat, standup, production)
 * - Auto-degrade indicator
 *
 * @example
 * ```tsx
 * <CoordinationStatus
 *   metrics={coordinationMetrics}
 *   onForceMode={forceMode}
 * />
 * ```
 */
export function CoordinationStatus({
  metrics,
  onForceMode,
  isLoading = false,
  compact = false,
  className = '',
}: CoordinationStatusProps): React.ReactElement {
  // No metrics available
  if (!metrics) {
    return (
      <div className={`bg-gray-100 dark:bg-gray-800 rounded-lg p-4 text-center ${className}`}>
        <p className="text-sm text-gray-500 dark:text-gray-400">
          Coordination metrics unavailable
        </p>
      </div>
    );
  }

  const overheadPercent = Math.round(metrics.overhead_percent * 10) / 10;
  const gaugeWidth = Math.min(metrics.overhead_percent, 100);
  const overheadColor = getOverheadColor(metrics.overhead_percent);
  const overheadTextColor = getOverheadTextColor(metrics.overhead_percent);

  // Compact mode: only mode badge and overhead percentage
  if (compact) {
    return (
      <div className={`flex items-center gap-3 ${className}`}>
        <ModeSelector
          currentMode={metrics.current_mode}
          onSelect={onForceMode}
          isLoading={isLoading}
        />

        <div className={`text-sm font-semibold ${overheadTextColor}`}>
          {overheadPercent}% overhead
        </div>
      </div>
    );
  }

  // Full mode: complete status bar
  return (
    <div className={`bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 shadow-sm ${className}`}>
      {/* Header: Mode badge + Auto-degrade indicator */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-3">
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Coordination Mode:
          </span>
          <ModeSelector
            currentMode={metrics.current_mode}
            onSelect={onForceMode}
            isLoading={isLoading}
          />
        </div>

        {/* Auto-degrade indicator */}
        <div className="flex items-center gap-2">
          <span className="text-xs text-gray-500 dark:text-gray-400">
            Auto-degrade:
          </span>
          <span
            className={`text-xs px-2 py-0.5 rounded ${
              metrics.auto_degrade_enabled
                ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300'
                : 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'
            }`}
          >
            {metrics.auto_degrade_enabled ? 'Enabled' : 'Manual'}
          </span>
        </div>
      </div>

      {/* Token overhead gauge */}
      <div className="mb-3">
        <div className="flex items-center justify-between mb-1">
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Token Overhead
          </span>
          <span className={`text-sm font-semibold ${overheadTextColor}`}>
            {overheadPercent}%
          </span>
        </div>

        {/* Gauge bar */}
        <div className="relative h-4 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
          {/* Filled portion */}
          <div
            className={`absolute left-0 top-0 h-full ${overheadColor} transition-all duration-300`}
            style={{ width: `${gaugeWidth}%` }}
          />

          {/* Threshold line at 30% */}
          <div
            className="absolute top-0 h-full w-0.5 bg-red-700 dark:bg-red-500"
            style={{ left: '30%' }}
          />
        </div>

        <div className="flex items-center justify-between mt-1">
          <span className="text-xs text-gray-500 dark:text-gray-400">0%</span>
          <span className="text-xs text-gray-500 dark:text-gray-400">
            Threshold: {metrics.max_overhead_percent}%
          </span>
          <span className="text-xs text-gray-500 dark:text-gray-400">100%</span>
        </div>
      </div>

      {/* Token breakdown */}
      <div className="flex items-center gap-4 text-xs text-gray-600 dark:text-gray-400">
        <div>
          <span className="font-medium">Heartbeat:</span>{' '}
          {formatTokenCount(metrics.heartbeat_tokens)}
        </div>
        <span className="text-gray-300 dark:text-gray-600">|</span>
        <div>
          <span className="font-medium">Standup:</span>{' '}
          {formatTokenCount(metrics.standup_tokens)}
        </div>
        <span className="text-gray-300 dark:text-gray-600">|</span>
        <div>
          <span className="font-medium">Production:</span>{' '}
          {formatTokenCount(metrics.production_tokens)}
        </div>
      </div>
    </div>
  );
}
