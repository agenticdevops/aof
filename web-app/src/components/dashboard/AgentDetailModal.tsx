import React from 'react'
import Modal from '@/components/common/Modal'
import Card from '@/components/common/Card'
import Badge from '@/components/common/Badge'
import Button from '@/components/common/Button'
import type { DashboardAgent } from '@/types/dashboard'
import { TrendingUp, TrendingDown, Activity, AlertCircle } from 'lucide-react'

interface AgentDetailModalProps {
  agent: DashboardAgent | null
  isOpen: boolean
  onClose: () => void
}

/**
 * Expanded agent detail modal
 * Shows comprehensive metrics and additional information
 */
export const AgentDetailModal: React.FC<AgentDetailModalProps> = ({
  agent,
  isOpen,
  onClose,
}) => {
  if (!agent) return null

  // Determine status configuration
  const getStatusConfig = () => {
    switch (agent.status) {
      case 'active':
        return {
          variant: 'success' as const,
          label: 'Active',
          color: 'text-green-600 dark:text-green-400',
        }
      case 'idle':
        return {
          variant: 'warning' as const,
          label: 'Idle',
          color: 'text-yellow-600 dark:text-yellow-400',
        }
      case 'error':
        return {
          variant: 'error' as const,
          label: 'Error',
          color: 'text-red-600 dark:text-red-400',
        }
    }
  }

  const statusConfig = getStatusConfig()

  // Calculate derived metrics
  const errorRate = (100 - agent.metrics.successRate).toFixed(1)
  const tasksPerDay = Math.round(agent.metrics.tasksCompleted / 7) // Assuming 7-day average

  // Determine trend indicators (placeholder logic)
  const uptimeTrend = agent.metrics.uptime >= 95 ? 'up' : 'down'
  const successRateTrend = agent.metrics.successRate >= 90 ? 'up' : 'down'

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={agent.name}
      size="lg"
      footer={
        <div className="flex gap-3 justify-end">
          <Button variant="secondary" onClick={onClose}>
            Close
          </Button>
          <Button variant="ghost">View Details</Button>
          <Button variant="secondary">Pause Agent</Button>
        </div>
      }
    >
      <div className="space-y-6">
        {/* Agent Header */}
        <Card elevation="flat">
          <div className="flex items-center gap-4">
            {/* Avatar */}
            <div
              className="flex-shrink-0 w-16 h-16 rounded-full flex items-center justify-center text-3xl"
              style={{ backgroundColor: agent.personaColor }}
            >
              {agent.personaIcon}
            </div>

            {/* Name + Role + Status */}
            <div className="flex-1">
              <h3 className="text-xl font-bold text-gray-900 dark:text-white">
                {agent.name}
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                {agent.role}
              </p>
            </div>

            {/* Status Badge */}
            <Badge variant={statusConfig.variant} size="md">
              {statusConfig.label}
            </Badge>
          </div>
        </Card>

        {/* Metrics Section - 2x2 Grid with Trends */}
        <Card elevation="flat">
          <h4 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-4">
            Performance Metrics
          </h4>
          <div className="grid grid-cols-2 gap-6">
            {/* Uptime */}
            <div>
              <div className="flex items-center justify-between mb-2">
                <p className="text-xs text-gray-500 dark:text-gray-400">Uptime</p>
                {uptimeTrend === 'up' ? (
                  <TrendingUp className="w-4 h-4 text-green-500" />
                ) : (
                  <TrendingDown className="w-4 h-4 text-red-500" />
                )}
              </div>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">
                {agent.metrics.uptime.toFixed(1)}%
              </p>
              {/* Progress bar */}
              <div className="mt-2 w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                <div
                  className="bg-green-500 h-2 rounded-full transition-all"
                  style={{ width: `${agent.metrics.uptime}%` }}
                />
              </div>
            </div>

            {/* Success Rate */}
            <div>
              <div className="flex items-center justify-between mb-2">
                <p className="text-xs text-gray-500 dark:text-gray-400">Success Rate</p>
                {successRateTrend === 'up' ? (
                  <TrendingUp className="w-4 h-4 text-green-500" />
                ) : (
                  <TrendingDown className="w-4 h-4 text-red-500" />
                )}
              </div>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">
                {agent.metrics.successRate.toFixed(1)}%
              </p>
              {/* Progress bar */}
              <div className="mt-2 w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                <div
                  className="bg-blue-500 h-2 rounded-full transition-all"
                  style={{ width: `${agent.metrics.successRate}%` }}
                />
              </div>
            </div>

            {/* Response Time */}
            <div>
              <div className="flex items-center justify-between mb-2">
                <p className="text-xs text-gray-500 dark:text-gray-400">Avg Response Time</p>
                <Activity className="w-4 h-4 text-blue-500" />
              </div>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">
                {agent.metrics.responseTime}ms
              </p>
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                Last 24 hours
              </p>
            </div>

            {/* Tasks Completed */}
            <div>
              <div className="flex items-center justify-between mb-2">
                <p className="text-xs text-gray-500 dark:text-gray-400">Tasks Completed</p>
                <Activity className="w-4 h-4 text-purple-500" />
              </div>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">
                {agent.metrics.tasksCompleted}
              </p>
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                ~{tasksPerDay}/day
              </p>
            </div>
          </div>
        </Card>

        {/* Additional Info Section */}
        <Card elevation="flat">
          <h4 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-4">
            Additional Information
          </h4>
          <div className="space-y-3">
            {/* Last Active */}
            <div className="flex justify-between">
              <span className="text-sm text-gray-600 dark:text-gray-400">Last Active</span>
              <span className="text-sm font-medium text-gray-900 dark:text-white">
                {new Date(agent.updatedAt).toLocaleString()}
              </span>
            </div>

            {/* Total Runs */}
            <div className="flex justify-between">
              <span className="text-sm text-gray-600 dark:text-gray-400">Total Runs</span>
              <span className="text-sm font-medium text-gray-900 dark:text-white">
                {agent.metrics.tasksCompleted}
              </span>
            </div>

            {/* Error Rate */}
            <div className="flex justify-between">
              <span className="text-sm text-gray-600 dark:text-gray-400">Error Rate</span>
              <span className={`text-sm font-medium ${
                parseFloat(errorRate) > 10 ? 'text-red-600 dark:text-red-400' : 'text-green-600 dark:text-green-400'
              }`}>
                {errorRate}%
                {parseFloat(errorRate) > 10 && (
                  <AlertCircle className="inline w-4 h-4 ml-1" />
                )}
              </span>
            </div>
          </div>
        </Card>
      </div>
    </Modal>
  )
}

export default AgentDetailModal
