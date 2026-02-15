import React from 'react'
import AgentCard from './AgentCard'
import EmptyState from '@/components/common/EmptyState'
import LoadingSpinner from '@/components/common/LoadingSpinner'
import type { DashboardAgent } from '@/types/dashboard'
import { FolderOpen } from 'lucide-react'

interface AgentGridProps {
  agents: DashboardAgent[]
  onAgentClick: (agent: DashboardAgent) => void
  isLoading?: boolean
}

/**
 * Responsive grid layout for Mission Control agents
 * - Mobile (< 768px): 1 column
 * - Tablet (768px - 1024px): 2 columns
 * - Desktop (> 1024px): 4 columns
 */
export const AgentGrid: React.FC<AgentGridProps> = ({
  agents,
  onAgentClick,
  isLoading = false,
}) => {
  // Loading state
  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-16">
        <LoadingSpinner size="lg" />
        <span className="ml-3 text-gray-600 dark:text-gray-400">Loading agents...</span>
      </div>
    )
  }

  // Empty state
  if (agents.length === 0) {
    return (
      <div className="flex items-center justify-center py-16">
        <EmptyState
          icon={<FolderOpen className="w-16 h-16" />}
          title="No agents yet"
          description="Create your first agent to get started"
        />
      </div>
    )
  }

  // Agent grid
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 px-6 py-8">
      {agents.map((agent) => (
        <AgentCard
          key={agent.id}
          agent={agent}
          onClick={onAgentClick}
        />
      ))}
    </div>
  )
}

export default AgentGrid
