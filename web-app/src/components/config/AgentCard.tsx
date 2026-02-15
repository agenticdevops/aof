import React from 'react'
import { Edit2, Trash2, CheckCircle } from 'lucide-react'
import Card from '@/components/common/Card'
import Badge from '@/components/common/Badge'
import Button from '@/components/common/Button'
import { Agent } from '@/types'

interface AgentCardProps {
  agent: Agent
  onEdit: (agent: Agent) => void
  onDelete: (id: string) => void
}

export const AgentCard: React.FC<AgentCardProps> = ({ agent, onEdit, onDelete }) => {
  return (
    <Card elevation="lifted">
      <div className="space-y-4">
        {/* Header */}
        <div className="flex items-start justify-between">
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">{agent.name}</h3>
            <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">{agent.model}</p>
          </div>
          <Badge variant="info" size="sm" className="capitalize">
            {agent.type}
          </Badge>
        </div>

        {/* Status */}
        <div className="flex items-center gap-2 text-sm">
          <CheckCircle className="w-4 h-4 text-green-500" />
          <span className="text-green-600 dark:text-green-400 font-medium">Healthy</span>
        </div>

        {/* Capabilities */}
        {agent.capabilities && agent.capabilities.length > 0 && (
          <div>
            <p className="text-xs text-gray-500 dark:text-gray-400 mb-2">Capabilities</p>
            <div className="flex flex-wrap gap-1">
              {agent.capabilities.slice(0, 3).map((cap) => (
                <Badge key={cap} variant="neutral" size="sm">
                  {cap}
                </Badge>
              ))}
              {agent.capabilities.length > 3 && (
                <Badge variant="neutral" size="sm">
                  +{agent.capabilities.length - 3}
                </Badge>
              )}
            </div>
          </div>
        )}

        {/* Actions */}
        <div className="flex gap-2 pt-2 border-t border-gray-200 dark:border-gray-700">
          <Button
            variant="secondary"
            size="sm"
            onClick={() => onEdit(agent)}
            icon={<Edit2 className="w-4 h-4" />}
            iconPosition="left"
            fullWidth
          >
            Edit
          </Button>
          <Button
            variant="danger"
            size="sm"
            onClick={() => onDelete(agent.id)}
            icon={<Trash2 className="w-4 h-4" />}
            iconPosition="left"
            fullWidth
          >
            Delete
          </Button>
        </div>
      </div>
    </Card>
  )
}

export default AgentCard
