import React from 'react'
import Card from '@/components/common/Card'
import Badge from '@/components/common/Badge'
import AgentAvatar from '@/components/common/AgentAvatar'
import type { DashboardAgent } from '@/types/dashboard'
import { mapAgentTypeToPersonaType } from '@/types/personas'
import { getPersonaBorderColor, getPersonaFont } from '@/utils/personaStyles'
import { useAppSelector } from '@/store/hooks'
import { selectIsDarkMode } from '@/store/slices/appSlice'

interface AgentCardProps {
  agent: DashboardAgent
  onClick: (agent: DashboardAgent) => void
}

/**
 * Agent card for Mission Control dashboard
 * Shows live status, metrics, and persona styling
 */
export const AgentCard: React.FC<AgentCardProps> = ({ agent, onClick }) => {
  const isDarkMode = useAppSelector(selectIsDarkMode)

  // Map agent role to persona type (using role as proxy for type)
  const personaType = mapAgentTypeToPersonaType(agent.role)

  // Get persona styling
  const borderColor = getPersonaBorderColor(personaType, isDarkMode)
  const fontClass = getPersonaFont(personaType)

  // Determine status badge variant and animation
  const getStatusConfig = () => {
    switch (agent.status) {
      case 'active':
        return {
          variant: 'success' as const,
          animationClass: 'animate-status-pulse-green',
          label: 'Active',
        }
      case 'idle':
        return {
          variant: 'warning' as const,
          animationClass: 'animate-status-pulse-yellow',
          label: 'Idle',
        }
      case 'error':
        return {
          variant: 'error' as const,
          animationClass: 'animate-status-pulse-red',
          label: 'Error',
        }
    }
  }

  const statusConfig = getStatusConfig()

  return (
    <Card
      elevation="lifted"
      clickable
      onClick={() => onClick(agent)}
      className={`status-transition scale-transition shadow-transition ${
        agent.status === 'active' ? 'animate-heartbeat' : ''
      } hover:scale-105 hover:shadow-lg cursor-pointer border-l-4 transition-all duration-200`}
      style={{ borderLeftColor: borderColor }}
    >
      <div className="space-y-4 p-6">
        {/* Header: Avatar + Name + Role + Status */}
        <div className="flex items-start gap-3">
          {/* Avatar with persona styling */}
          <AgentAvatar
            personaType={personaType}
            icon={agent.personaIcon}
            size="md"
            isOnline={agent.status === 'active'}
            isDarkMode={isDarkMode}
          />

          {/* Name + Role */}
          <div className="flex-1 min-w-0">
            <h3
              className={`text-lg ${fontClass} truncate`}
              style={{ color: borderColor }}
            >
              {agent.name}
            </h3>
            <p className="text-sm text-gray-600 dark:text-gray-400 truncate">
              {agent.role}
            </p>
          </div>

          {/* Status Badge */}
          <div className={statusConfig.animationClass}>
            <Badge variant={statusConfig.variant} size="sm">
              {statusConfig.label}
            </Badge>
          </div>
        </div>

        {/* Metrics Grid - 2x2 */}
        <div className="grid grid-cols-2 gap-4">
          {/* Uptime */}
          <div>
            <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">Uptime</p>
            <p className="text-lg font-semibold text-gray-900 dark:text-white">
              {agent.metrics.uptime.toFixed(1)}%
            </p>
          </div>

          {/* Success Rate */}
          <div>
            <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">Success Rate</p>
            <p className="text-lg font-semibold text-gray-900 dark:text-white">
              {agent.metrics.successRate.toFixed(1)}%
            </p>
          </div>

          {/* Response Time */}
          <div>
            <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">Response</p>
            <p className="text-lg font-semibold text-gray-900 dark:text-white">
              {agent.metrics.responseTime}ms
            </p>
          </div>

          {/* Tasks Completed */}
          <div>
            <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">Tasks</p>
            <p className="text-lg font-semibold text-gray-900 dark:text-white">
              {agent.metrics.tasksCompleted}
            </p>
          </div>
        </div>
      </div>
    </Card>
  )
}

export default AgentCard
