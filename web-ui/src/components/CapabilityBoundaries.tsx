/**
 * CapabilityBoundaries component - expandable CAN/CANNOT section.
 * Shows agent capability boundaries as collapsible lists with
 * green (CAN) and red (CANNOT) color coding.
 */

import React, { useState } from 'react';

/**
 * Component props.
 */
export interface CapabilityBoundariesProps {
  /** Actions the agent CAN perform */
  can: string[];

  /** Actions the agent CANNOT perform */
  cannot: string[];

  /** Optional className for container styling */
  className?: string;
}

/**
 * Chevron icon component that rotates when expanded.
 */
function ChevronIcon({ expanded }: { expanded: boolean }): React.ReactElement {
  return (
    <svg
      className={`w-4 h-4 transition-transform duration-200 ${expanded ? 'rotate-180' : ''}`}
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
      strokeWidth={2}
      aria-hidden="true"
    >
      <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
    </svg>
  );
}

/**
 * CapabilityBoundaries component.
 *
 * Renders a collapsible section showing what an agent CAN and CANNOT do.
 * Initially collapsed. Clicking the header expands/collapses the lists.
 *
 * @example
 * ```tsx
 * <CapabilityBoundaries
 *   can={["kubectl operations", "pod debugging", "log analysis"]}
 *   cannot={["modify cluster RBAC", "delete PVs without approval"]}
 * />
 * ```
 */
export function CapabilityBoundaries({
  can,
  cannot,
  className = '',
}: CapabilityBoundariesProps): React.ReactElement | null {
  const [expanded, setExpanded] = useState(false);

  // Don't render if both arrays are empty
  if ((!can || can.length === 0) && (!cannot || cannot.length === 0)) {
    return null;
  }

  const handleToggle = () => {
    setExpanded((prev) => !prev);
  };

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleToggle();
    }
  };

  return (
    <div
      className={`border border-gray-200 dark:border-gray-600 rounded-lg overflow-hidden ${className}`}
    >
      {/* Header (clickable to toggle) */}
      <button
        type="button"
        className="w-full flex items-center justify-between px-3 py-2 bg-gray-50 dark:bg-gray-700/50 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors cursor-pointer"
        onClick={handleToggle}
        onKeyDown={handleKeyDown}
        aria-expanded={expanded}
        aria-controls="capability-content"
      >
        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
          Capabilities
        </span>
        <ChevronIcon expanded={expanded} />
      </button>

      {/* Collapsible content */}
      {expanded && (
        <div
          id="capability-content"
          className="px-3 py-2 bg-gray-50/50 dark:bg-gray-800/50"
        >
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {/* CAN section */}
            {can && can.length > 0 && (
              <div>
                <h4 className="text-xs font-semibold text-green-700 dark:text-green-400 uppercase tracking-wide mb-1.5">
                  I CAN:
                </h4>
                <ul className="space-y-1" role="list" aria-label="Agent capabilities">
                  {can.map((item, index) => (
                    <li
                      key={index}
                      className="flex items-start gap-1.5 text-xs text-gray-700 dark:text-gray-300"
                    >
                      <span
                        className="mt-1 w-1.5 h-1.5 rounded-full bg-green-500 flex-shrink-0"
                        aria-hidden="true"
                      />
                      <span className="break-words">{item}</span>
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {/* CANNOT section */}
            {cannot && cannot.length > 0 && (
              <div>
                <h4 className="text-xs font-semibold text-red-700 dark:text-red-400 uppercase tracking-wide mb-1.5">
                  I CANNOT:
                </h4>
                <ul className="space-y-1" role="list" aria-label="Agent limitations">
                  {cannot.map((item, index) => (
                    <li
                      key={index}
                      className="flex items-start gap-1.5 text-xs text-gray-700 dark:text-gray-300"
                    >
                      <span
                        className="mt-1 w-1.5 h-1.5 rounded-full bg-red-500 flex-shrink-0"
                        aria-hidden="true"
                      />
                      <span className="break-words">{item}</span>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
