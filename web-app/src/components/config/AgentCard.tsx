import React from 'react'
import { Agent } from '@/types'
import Card from '@/components/common/Card'
import Badge from '@/components/common/Badge'
import Button from '@/components/common/Button'

interface AgentCardProps {
  agent: Agent
  onEdit: (agent: Agent) => void
  onDelete: (id: string) => void
}

export const AgentCard: React.FC<AgentCardProps> = ({ agent, onEdit, onDelete }) => {
  return (
    <Card elevation="lifted" hoverable>
      <div className="space-y-4">
        <div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">{agent.name}</h3>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">{agent.model}</p>
        </div>

        <div className="flex flex-wrap gap-2">
          <Badge variant="info" size="sm">
            {agent.type}
          </Badge>
          <Badge variant="success" size="sm">
            ✓ Healthy
          </Badge>
        </div>

        {agent.capabilities && agent.capabilities.length > 0 && (
          <div>
            <p className="text-xs text-gray-600 dark:text-gray-400 font-medium mb-2">Capabilities</p>
            <p className="text-sm text-gray-700 dark:text-gray-300">{agent.capabilities.join(', ')}</p>
          </div>
        )}

        {agent.instructions && (
          <div>
            <p className="text-xs text-gray-600 dark:text-gray-400 font-medium mb-1">Instructions</p>
            <p className="text-sm text-gray-700 dark:text-gray-300 line-clamp-2">{agent.instructions}</p>
          </div>
        )}

        <div className="flex gap-2 pt-4 border-t border-gray-200 dark:border-gray-700">
          <Button
            variant="secondary"
            size="sm"
            fullWidth
            onClick={() => onEdit(agent)}
          >
            Edit
          </Button>
          <Button
            variant="danger"
            size="sm"
            fullWidth
            onClick={() => onDelete(agent.id)}
          >
            Delete
          </Button>
        </div>
      </div>
    </Card>
  )
}

export default AgentCard
