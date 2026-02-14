/**
 * Status indicator component for agent/connection status.
 * Uses color coding: green (connected/idle), yellow (working/reconnecting), red (error/disconnected).
 */

import React from 'react';
import type { AgentStatus } from '../types/events';

/**
 * Component props.
 */
interface StatusIndicatorProps {
  /** Status type */
  status: 'connected' | 'disconnected' | 'reconnecting' | AgentStatus;

  /** Optional label text */
  label?: string;

  /** Optional className for styling */
  className?: string;
}

/**
 * Map status to color classes.
 */
function getStatusColor(status: StatusIndicatorProps['status']): string {
  switch (status) {
    case 'connected':
    case 'idle':
      return 'bg-green-500';

    case 'reconnecting':
    case 'working':
      return 'bg-yellow-500';

    case 'disconnected':
    case 'error':
    case 'blocked':
      return 'bg-red-500';

    default:
      return 'bg-gray-500';
  }
}

/**
 * Status indicator component.
 */
export function StatusIndicator({
  status,
  label,
  className = '',
}: StatusIndicatorProps): React.ReactElement {
  const colorClass = getStatusColor(status);

  return (
    <div className={`flex items-center gap-2 ${className}`}>
      <div className={`w-3 h-3 rounded-full ${colorClass}`} />
      {label && <span className="text-sm">{label}</span>}
    </div>
  );
}
