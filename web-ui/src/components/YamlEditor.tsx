/**
 * YamlEditor - Simple YAML/Markdown editor for modifying generated files.
 * Styled textarea with line numbers and monospace font.
 */

import { useState, useCallback } from 'react';

interface YamlEditorProps {
  /** File path being edited */
  filePath: string;

  /** Initial file content */
  initialContent: string;

  /** Callback when save is clicked */
  onSave: (filePath: string, content: string) => void;

  /** Callback when cancel is clicked */
  onCancel: () => void;
}

/**
 * Simple YAML/Markdown editor component.
 */
export function YamlEditor({
  filePath,
  initialContent,
  onSave,
  onCancel,
}: YamlEditorProps) {
  const [content, setContent] = useState(initialContent);
  const [hasChanges, setHasChanges] = useState(false);

  /**
   * Handle content change.
   */
  const handleChange = useCallback((value: string) => {
    setContent(value);
    setHasChanges(value !== initialContent);
  }, [initialContent]);

  /**
   * Handle save action.
   */
  const handleSave = useCallback(() => {
    onSave(filePath, content);
  }, [filePath, content, onSave]);

  const lineCount = content.split('\n').length;

  return (
    <div className="flex flex-col h-full bg-gray-900">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 bg-gray-800 border-b border-gray-700">
        <div>
          <h3 className="text-sm font-semibold text-white">Edit File</h3>
          <p className="text-xs text-gray-400 mt-1">{filePath}</p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={onCancel}
            className="px-4 py-2 bg-gray-700 text-white rounded hover:bg-gray-600 transition-colors text-sm"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={!hasChanges}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors text-sm"
          >
            Save Changes
          </button>
        </div>
      </div>

      {/* Editor */}
      <div className="flex-1 flex overflow-hidden">
        {/* Line numbers */}
        <div className="bg-gray-800 px-3 py-4 text-right text-xs text-gray-500 font-mono select-none overflow-y-auto">
          {Array.from({ length: lineCount }, (_, i) => (
            <div key={i + 1} className="leading-6">
              {i + 1}
            </div>
          ))}
        </div>

        {/* Content */}
        <textarea
          value={content}
          onChange={(e) => handleChange(e.target.value)}
          className="flex-1 px-4 py-4 bg-gray-900 text-gray-100 font-mono text-sm resize-none focus:outline-none leading-6"
          spellCheck={false}
        />
      </div>

      {/* Status bar */}
      <div className="px-4 py-2 bg-gray-800 border-t border-gray-700 text-xs text-gray-400">
        {hasChanges ? (
          <span className="text-yellow-500">● Unsaved changes</span>
        ) : (
          <span>No changes</span>
        )}
        <span className="ml-4">{lineCount} lines</span>
      </div>
    </div>
  );
}
