import React, { useEffect } from 'react'
import { useAppDispatch, useAppSelector } from '@/store'
import { setDashboardAgents, setDashboardSelectedAgent } from '@/store'
import AgentGrid from '@/components/dashboard/AgentGrid'
import AgentDetailModal from '@/components/dashboard/AgentDetailModal'
import type { DashboardAgent } from '@/types/dashboard'

/**
 * Mission Control Dashboard
 * Central monitoring interface for agent squad
 */
const MissionControl: React.FC = () => {
  const dispatch = useAppDispatch()
  const { agents, selectedAgent, isLoading } = useAppSelector((state) => state.dashboard)

  // Load mock agent data on mount
  // TODO: Replace with WebSocket integration in Plan 03
  useEffect(() => {
    const mockAgents: DashboardAgent[] = [
      {
        id: '1',
        name: 'Xops',
        role: 'Orchestrator',
        status: 'active',
        metrics: {
          uptime: 98.5,
          successRate: 94.2,
          responseTime: 245,
          tasksCompleted: 127,
        },
        personaColor: '#10b981', // emerald-500
        personaIcon: '⚙️',
        updatedAt: new Date(),
      },
      {
        id: '2',
        name: 'K8sOps',
        role: 'Kubernetes Specialist',
        status: 'active',
        metrics: {
          uptime: 99.1,
          successRate: 97.8,
          responseTime: 312,
          tasksCompleted: 89,
        },
        personaColor: '#3b82f6', // blue-500
        personaIcon: '☸️',
        updatedAt: new Date(),
      },
      {
        id: '3',
        name: 'SREWatch',
        role: 'Observability Agent',
        status: 'idle',
        metrics: {
          uptime: 95.3,
          successRate: 91.5,
          responseTime: 428,
          tasksCompleted: 56,
        },
        personaColor: '#8b5cf6', // violet-500
        personaIcon: '👁️',
        updatedAt: new Date(),
      },
      {
        id: '4',
        name: 'InfraBot',
        role: 'Infrastructure Agent',
        status: 'active',
        metrics: {
          uptime: 97.8,
          successRate: 93.1,
          responseTime: 356,
          tasksCompleted: 73,
        },
        personaColor: '#f59e0b', // amber-500
        personaIcon: '🏗️',
        updatedAt: new Date(),
      },
      {
        id: '5',
        name: 'ErrorTracer',
        role: 'Debugging Specialist',
        status: 'error',
        metrics: {
          uptime: 82.4,
          successRate: 76.3,
          responseTime: 891,
          tasksCompleted: 34,
        },
        personaColor: '#ef4444', // red-500
        personaIcon: '🔍',
        updatedAt: new Date(),
      },
    ]

    dispatch(setDashboardAgents(mockAgents))
  }, [dispatch])

  // Handle agent card click
  const handleAgentClick = (agent: DashboardAgent) => {
    dispatch(setDashboardSelectedAgent(agent))
  }

  // Handle modal close
  const handleModalClose = () => {
    dispatch(setDashboardSelectedAgent(null))
  }

  return (
    <div className="min-h-screen bg-white dark:bg-gray-900">
      {/* Header */}
      <header className="border-b border-gray-200 dark:border-gray-800 py-6">
        <div className="px-6">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
            Mission Control
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-2">
            Monitor your agent squad in real-time
          </p>
        </div>
      </header>

      {/* Main Content */}
      <main className="px-6 py-8">
        <AgentGrid
          agents={agents}
          onAgentClick={handleAgentClick}
          isLoading={isLoading}
        />
      </main>

      {/* Agent Detail Modal */}
      <AgentDetailModal
        agent={selectedAgent}
        isOpen={selectedAgent !== null}
        onClose={handleModalClose}
      />
    </div>
  )
}

export default MissionControl
