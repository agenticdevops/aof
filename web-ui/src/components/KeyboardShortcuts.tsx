/**
 * Keyboard shortcuts help modal.
 * Documents keyboard navigation for Kanban board accessibility.
 */

import React from 'react';

/**
 * Component props.
 */
export interface KeyboardShortcutsProps {
  /** Whether modal is visible */
  isOpen: boolean;

  /** Close handler */
  onClose: () => void;
}

/**
 * Keyboard shortcuts documentation component.
 *
 * Shortcuts:
 * - Tab: Navigate between tasks
 * - Space: Pick up/drop task
 * - Arrow keys: Move task within lane or between lanes
 * - Escape: Cancel drag operation
 * - Enter: Open task details
 *
 * @example
 * ```tsx
 * const [showHelp, setShowHelp] = useState(false);
 * <KeyboardShortcuts isOpen={showHelp} onClose={() => setShowHelp(false)} />
 * ```
 */
export function KeyboardShortcuts({
  isOpen,
  onClose,
}: KeyboardShortcutsProps): React.ReactElement | null {
  if (!isOpen) return null;

  const shortcuts = [
    { key: 'Tab', description: 'Navigate between tasks' },
    { key: 'Space', description: 'Pick up or drop task (drag mode)' },
    { key: 'Arrow Keys', description: 'Move task within lane or between lanes' },
    { key: 'Escape', description: 'Cancel drag operation' },
    { key: 'Enter', description: 'Open task details' },
    { key: '?', description: 'Show keyboard shortcuts' },
  ];

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50"
      role="dialog"
      aria-labelledby="shortcuts-title"
      aria-modal="true"
    >
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-2xl max-w-md w-full p-6">
        {/* Header */}
        <div className="flex items-center justify-between mb-4">
          <h2
            id="shortcuts-title"
            className="text-xl font-semibold text-gray-900 dark:text-gray-100"
          >
            Keyboard Shortcuts
          </h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
            aria-label="Close keyboard shortcuts"
          >
            <svg
              className="w-6 h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        {/* Shortcuts list */}
        <div className="space-y-3">
          {shortcuts.map((shortcut) => (
            <div
              key={shortcut.key}
              className="flex items-center justify-between py-2 border-b border-gray-200 dark:border-gray-700 last:border-0"
            >
              <span className="text-sm text-gray-600 dark:text-gray-400">
                {shortcut.description}
              </span>
              <kbd className="px-3 py-1 text-sm font-semibold bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200 rounded border border-gray-300 dark:border-gray-600">
                {shortcut.key}
              </kbd>
            </div>
          ))}
        </div>

        {/* Footer */}
        <div className="mt-6 pt-4 border-t border-gray-200 dark:border-gray-700">
          <p className="text-xs text-gray-500 dark:text-gray-400 text-center">
            Keyboard navigation powered by dnd-kit
          </p>
        </div>
      </div>
    </div>
  );
}
