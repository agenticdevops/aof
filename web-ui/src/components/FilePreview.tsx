/**
 * FilePreview - Modal/panel showing generated files with preview and edit capability.
 * Displays syntax-highlighted YAML/Markdown with confirm/edit/cancel actions.
 */

import { useState, useCallback } from 'react';
import { YamlEditor } from './YamlEditor';
import { useConversation } from '../hooks/useConversation';

interface FilePreviewProps {
  /** Generated files (path -> content) */
  files: Record<string, string>;

  /** Summary message for the files */
  summary?: string;
}

/**
 * File preview modal component.
 */
export function FilePreview({ files, summary }: FilePreviewProps) {
  const { confirm, cancel } = useConversation();
  const [isEditing, setIsEditing] = useState(false);
  const [editingFile, setEditingFile] = useState<string | null>(null);
  const [modifiedFiles, setModifiedFiles] = useState<Record<string, string>>(files);

  const filePaths = Object.keys(modifiedFiles);
  const [selectedFile, setSelectedFile] = useState(filePaths[0] || '');

  /**
   * Handle edit button click.
   */
  const handleEdit = useCallback(() => {
    setEditingFile(selectedFile);
    setIsEditing(true);
  }, [selectedFile]);

  /**
   * Handle save from editor.
   */
  const handleSave = useCallback((filePath: string, content: string) => {
    setModifiedFiles((prev) => ({ ...prev, [filePath]: content }));
    setIsEditing(false);
    setEditingFile(null);
  }, []);

  /**
   * Handle cancel edit.
   */
  const handleCancelEdit = useCallback(() => {
    setIsEditing(false);
    setEditingFile(null);
  }, []);

  /**
   * Handle confirm (persist files).
   */
  const handleConfirm = useCallback(() => {
    confirm();
  }, [confirm]);

  /**
   * Handle cancel (discard files).
   */
  const handleCancel = useCallback(() => {
    cancel();
  }, [cancel]);

  // Show editor if editing
  if (isEditing && editingFile) {
    return (
      <YamlEditor
        filePath={editingFile}
        initialContent={modifiedFiles[editingFile]}
        onSave={handleSave}
        onCancel={handleCancelEdit}
      />
    );
  }

  // Show preview
  return (
    <div className="flex flex-col h-full bg-gray-900">
      {/* Header */}
      <div className="px-4 py-3 bg-gray-800 border-b border-gray-700">
        <h3 className="text-lg font-semibold text-white">Generated Files</h3>
        {summary && <p className="text-sm text-gray-400 mt-1">{summary}</p>}
      </div>

      {/* File tabs */}
      {filePaths.length > 1 && (
        <div className="flex gap-2 px-4 py-2 bg-gray-800 border-b border-gray-700 overflow-x-auto">
          {filePaths.map((path) => (
            <button
              key={path}
              onClick={() => setSelectedFile(path)}
              className={`px-3 py-1 rounded text-sm transition-colors ${
                selectedFile === path
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              }`}
            >
              {path.split('/').pop()}
            </button>
          ))}
        </div>
      )}

      {/* File preview */}
      <div className="flex-1 overflow-y-auto p-4">
        <div className="max-w-4xl mx-auto">
          <div className="bg-gray-800 rounded-lg overflow-hidden">
            {/* File path header */}
            <div className="px-4 py-2 bg-gray-700 border-b border-gray-600">
              <span className="text-xs text-gray-300 font-mono">{selectedFile}</span>
            </div>

            {/* File content */}
            <pre className="p-4 text-sm text-gray-100 font-mono overflow-x-auto">
              {modifiedFiles[selectedFile]}
            </pre>
          </div>
        </div>
      </div>

      {/* Action buttons */}
      <div className="flex justify-end gap-3 px-4 py-3 bg-gray-800 border-t border-gray-700">
        <button
          onClick={handleCancel}
          className="px-4 py-2 bg-gray-700 text-white rounded hover:bg-gray-600 transition-colors"
        >
          Cancel
        </button>
        <button
          onClick={handleEdit}
          className="px-4 py-2 bg-gray-700 text-white rounded hover:bg-gray-600 transition-colors"
        >
          Edit Before Saving
        </button>
        <button
          onClick={handleConfirm}
          className="px-6 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors font-semibold"
        >
          Confirm & Save
        </button>
      </div>
    </div>
  );
}
