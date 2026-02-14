/**
 * CreateAgent page - conversational agent creation interface.
 */

import React from 'react';
import { ConversationPanel } from '../components/ConversationPanel';
import { FilePreview } from '../components/FilePreview';
import { useConversation } from '../hooks/useConversation';

/**
 * CreateAgent page component.
 */
export function CreateAgent(): React.ReactElement {
  const { pendingFiles } = useConversation();

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Page Header */}
      <div className="px-6 py-4 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-semibold text-white">Create Agent</h2>
            <p className="text-sm text-gray-400 mt-1">
              Describe your agent in natural language
            </p>
          </div>
          <a
            href="#/"
            className="text-sm text-blue-400 hover:text-blue-300 transition-colors"
          >
            ← Back to Dashboard
          </a>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-hidden">
        {pendingFiles ? (
          <FilePreview
            files={pendingFiles}
            summary="Review the generated files before saving"
          />
        ) : (
          <ConversationPanel />
        )}
      </div>
    </div>
  );
}
