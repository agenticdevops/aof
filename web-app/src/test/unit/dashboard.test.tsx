import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { Provider } from 'react-redux'
import { configureStore } from '@reduxjs/toolkit'
import AgentCard from '@/components/dashboard/AgentCard'
import AgentGrid from '@/components/dashboard/AgentGrid'
import AgentDetailModal from '@/components/dashboard/AgentDetailModal'
import dashboardReducer from '@/store/slices/dashboardSlice'
import type { DashboardAgent } from '@/types/dashboard'

// Mock agent data
const mockAgent: DashboardAgent = {
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
  personaColor: '#10b981',
  personaIcon: '⚙️',
  updatedAt: new Date('2024-01-01T12:00:00Z'),
}

const mockIdleAgent: DashboardAgent = {
  ...mockAgent,
  id: '2',
  name: 'IdleBot',
  status: 'idle',
}

const mockErrorAgent: DashboardAgent = {
  ...mockAgent,
  id: '3',
  name: 'ErrorBot',
  status: 'error',
}

// Test store setup
const createTestStore = () => {
  return configureStore({
    reducer: {
      dashboard: dashboardReducer,
    },
  })
}

describe('AgentCard Component', () => {
  it('renders agent name, role, status, and avatar', () => {
    const onClick = vi.fn()
    render(<AgentCard agent={mockAgent} onClick={onClick} />)

    expect(screen.getByText('Xops')).toBeInTheDocument()
    expect(screen.getByText('Orchestrator')).toBeInTheDocument()
    expect(screen.getByText('Active')).toBeInTheDocument()
    expect(screen.getByText('⚙️')).toBeInTheDocument()
  })

  it('displays all 4 metrics correctly', () => {
    const onClick = vi.fn()
    render(<AgentCard agent={mockAgent} onClick={onClick} />)

    expect(screen.getByText('98.5%')).toBeInTheDocument()
    expect(screen.getByText('94.2%')).toBeInTheDocument()
    expect(screen.getByText('245ms')).toBeInTheDocument()
    expect(screen.getByText('127')).toBeInTheDocument()
  })

  it('shows green status badge for active agents', () => {
    const onClick = vi.fn()
    const { container } = render(<AgentCard agent={mockAgent} onClick={onClick} />)

    // Check for status badge
    const badge = screen.getByText('Active')
    expect(badge).toBeInTheDocument()

    // Check for animate-status-pulse-green class
    const pulsingElement = container.querySelector('.animate-status-pulse-green')
    expect(pulsingElement).toBeInTheDocument()
  })

  it('shows yellow status badge for idle agents', () => {
    const onClick = vi.fn()
    const { container } = render(<AgentCard agent={mockIdleAgent} onClick={onClick} />)

    const badge = screen.getByText('Idle')
    expect(badge).toBeInTheDocument()

    const pulsingElement = container.querySelector('.animate-status-pulse-yellow')
    expect(pulsingElement).toBeInTheDocument()
  })

  it('shows red status badge for error agents', () => {
    const onClick = vi.fn()
    const { container } = render(<AgentCard agent={mockErrorAgent} onClick={onClick} />)

    const badge = screen.getByText('Error')
    expect(badge).toBeInTheDocument()

    const pulsingElement = container.querySelector('.animate-status-pulse-red')
    expect(pulsingElement).toBeInTheDocument()
  })

  it('applies heartbeat animation to active agents only', () => {
    const onClick = vi.fn()
    const { container, rerender } = render(
      <AgentCard agent={mockAgent} onClick={onClick} />
    )

    // Active agent should have heartbeat
    let card = container.querySelector('.animate-heartbeat')
    expect(card).toBeInTheDocument()

    // Idle agent should NOT have heartbeat
    rerender(<AgentCard agent={mockIdleAgent} onClick={onClick} />)
    card = container.querySelector('.animate-heartbeat')
    expect(card).not.toBeInTheDocument()
  })

  it('calls onClick handler when clicked', () => {
    const onClick = vi.fn()
    render(<AgentCard agent={mockAgent} onClick={onClick} />)

    const card = screen.getByText('Xops').closest('.cursor-pointer')
    expect(card).toBeInTheDocument()

    if (card) {
      fireEvent.click(card)
      expect(onClick).toHaveBeenCalledWith(mockAgent)
    }
  })
})

describe('AgentGrid Component', () => {
  it('renders grid with correct number of columns on desktop', () => {
    const onAgentClick = vi.fn()
    const { container } = render(
      <AgentGrid agents={[mockAgent]} onAgentClick={onAgentClick} />
    )

    // Check for responsive grid classes
    const grid = container.querySelector('.grid')
    expect(grid).toHaveClass('grid-cols-1', 'md:grid-cols-2', 'lg:grid-cols-4')
  })

  it('maps agents to AgentCard components', () => {
    const onAgentClick = vi.fn()
    const agents = [mockAgent, mockIdleAgent, mockErrorAgent]

    render(<AgentGrid agents={agents} onAgentClick={onAgentClick} />)

    expect(screen.getByText('Xops')).toBeInTheDocument()
    expect(screen.getByText('IdleBot')).toBeInTheDocument()
    expect(screen.getByText('ErrorBot')).toBeInTheDocument()
  })

  it('shows empty state when agents array is empty', () => {
    const onAgentClick = vi.fn()
    render(<AgentGrid agents={[]} onAgentClick={onAgentClick} />)

    expect(screen.getByText('No agents yet')).toBeInTheDocument()
    expect(screen.getByText('Create your first agent to get started')).toBeInTheDocument()
  })

  it('shows loading spinner when isLoading is true', () => {
    const onAgentClick = vi.fn()
    render(<AgentGrid agents={[]} onAgentClick={onAgentClick} isLoading={true} />)

    expect(screen.getByText('Loading agents...')).toBeInTheDocument()
  })
})

describe('AgentDetailModal Component', () => {
  it('renders nothing when isOpen is false', () => {
    const onClose = vi.fn()
    const { container } = render(
      <AgentDetailModal agent={mockAgent} isOpen={false} onClose={onClose} />
    )

    // Modal should not be in DOM when closed
    expect(container.querySelector('[role="dialog"]')).not.toBeInTheDocument()
  })

  it('renders modal with agent info when isOpen is true', () => {
    const onClose = vi.fn()
    render(
      <AgentDetailModal agent={mockAgent} isOpen={true} onClose={onClose} />
    )

    // Check for agent name in modal (appears multiple times: title + body)
    const agentNames = screen.getAllByText('Xops')
    expect(agentNames.length).toBeGreaterThan(0)
    expect(screen.getByText('Orchestrator')).toBeInTheDocument()
  })

  it('displays all metrics in detail modal', () => {
    const onClose = vi.fn()
    render(
      <AgentDetailModal agent={mockAgent} isOpen={true} onClose={onClose} />
    )

    // Check for uptime and success rate with decimal precision (may appear multiple times)
    const uptimeElements = screen.getAllByText('98.5%')
    expect(uptimeElements.length).toBeGreaterThan(0)

    const successElements = screen.getAllByText('94.2%')
    expect(successElements.length).toBeGreaterThan(0)

    expect(screen.getByText('245ms')).toBeInTheDocument()

    // Tasks completed appears twice (in metrics grid and additional info)
    const tasksElements = screen.getAllByText('127')
    expect(tasksElements.length).toBeGreaterThan(0)
  })

  it('calls onClose when close button is clicked', () => {
    const onClose = vi.fn()
    render(
      <AgentDetailModal agent={mockAgent} isOpen={true} onClose={onClose} />
    )

    const closeButton = screen.getByText('Close')
    fireEvent.click(closeButton)

    expect(onClose).toHaveBeenCalled()
  })

  it('shows error rate in additional info section', () => {
    const onClose = vi.fn()
    render(
      <AgentDetailModal agent={mockAgent} isOpen={true} onClose={onClose} />
    )

    // Error rate = 100 - 94.2 = 5.8%
    expect(screen.getByText('5.8%')).toBeInTheDocument()
  })
})
