/**
 * Persona Component Tests
 *
 * Tests for AgentAvatar, AgentCard, and MessageCard with persona styling.
 */

import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { Provider } from 'react-redux'
import { configureStore } from '@reduxjs/toolkit'
import AgentAvatar from '@/components/common/AgentAvatar'
import AgentCard from '@/components/dashboard/AgentCard'
import MessageCard from '@/components/chat/MessageCard'
import type { DashboardAgent } from '@/types/dashboard'
import type { ChatMessage } from '@/components/chat/MessageCard'
import appReducer from '@/store/slices/appSlice'
import dashboardReducer from '@/store/slices/dashboardSlice'

// Helper to create test store
function createTestStore() {
  return configureStore({
    reducer: {
      app: appReducer,
      dashboard: dashboardReducer,
    },
  })
}

describe('AgentAvatar Component', () => {
  it('renders with correct persona icon', () => {
    render(<AgentAvatar personaType="analyst" />)
    expect(screen.getByRole('img')).toHaveTextContent('📊')
  })

  it('renders custom icon when provided', () => {
    render(<AgentAvatar personaType="analyst" icon="🎯" />)
    expect(screen.getByRole('img')).toHaveTextContent('🎯')
  })

  it('shows online indicator when isOnline=true', () => {
    render(<AgentAvatar personaType="analyst" isOnline={true} />)
    expect(screen.getByRole('status')).toBeInTheDocument()
  })

  it('hides online indicator when isOnline=false', () => {
    render(<AgentAvatar personaType="analyst" isOnline={false} />)
    expect(screen.queryByRole('status')).not.toBeInTheDocument()
  })

  it('applies correct size class for sm variant', () => {
    const { container } = render(<AgentAvatar personaType="analyst" size="sm" />)
    const avatar = container.querySelector('[role="img"]')
    expect(avatar).toHaveClass('w-8', 'h-8', 'text-xs')
  })

  it('applies correct size class for md variant', () => {
    const { container } = render(<AgentAvatar personaType="analyst" size="md" />)
    const avatar = container.querySelector('[role="img"]')
    expect(avatar).toHaveClass('w-12', 'h-12', 'text-lg')
  })

  it('applies correct size class for lg variant', () => {
    const { container } = render(<AgentAvatar personaType="analyst" size="lg" />)
    const avatar = container.querySelector('[role="img"]')
    expect(avatar).toHaveClass('w-16', 'h-16', 'text-2xl')
  })

  it('applies correct background color for analyst persona', () => {
    const { container } = render(<AgentAvatar personaType="analyst" />)
    const avatar = container.querySelector('[role="img"]')
    expect(avatar).toHaveStyle({ backgroundColor: '#3b82f6' })
  })

  it('applies correct background color for coordinator persona', () => {
    const { container } = render(<AgentAvatar personaType="coordinator" />)
    const avatar = container.querySelector('[role="img"]')
    expect(avatar).toHaveStyle({ backgroundColor: '#10b981' })
  })
})

describe('AgentCard Component', () => {
  const mockAgent: DashboardAgent = {
    id: 'agent-1',
    name: 'Analytics Bot',
    role: 'Analyst',
    status: 'active',
    metrics: {
      uptime: 99.5,
      successRate: 95.2,
      responseTime: 150,
      tasksCompleted: 42,
    },
    personaColor: '#3b82f6',
    personaIcon: '📊',
    updatedAt: new Date(),
  }

  it('renders agent name with persona styling', () => {
    const store = createTestStore()
    render(
      <Provider store={store}>
        <AgentCard agent={mockAgent} onClick={() => {}} />
      </Provider>
    )
    expect(screen.getByText('Analytics Bot')).toBeInTheDocument()
  })

  it('displays persona icon via AgentAvatar', () => {
    const store = createTestStore()
    render(
      <Provider store={store}>
        <AgentCard agent={mockAgent} onClick={() => {}} />
      </Provider>
    )
    expect(screen.getByRole('img')).toHaveTextContent('📊')
  })

  it('shows online indicator for active agent', () => {
    const store = createTestStore()
    render(
      <Provider store={store}>
        <AgentCard agent={mockAgent} onClick={() => {}} />
      </Provider>
    )
    expect(screen.getByRole('status')).toBeInTheDocument()
  })

  it('does not show online indicator for idle agent', () => {
    const store = createTestStore()
    const idleAgent = { ...mockAgent, status: 'idle' as const }
    render(
      <Provider store={store}>
        <AgentCard agent={idleAgent} onClick={() => {}} />
      </Provider>
    )
    expect(screen.queryByRole('status')).not.toBeInTheDocument()
  })

  it('displays correct status badge for active agent', () => {
    const store = createTestStore()
    render(
      <Provider store={store}>
        <AgentCard agent={mockAgent} onClick={() => {}} />
      </Provider>
    )
    expect(screen.getByText('Active')).toBeInTheDocument()
  })

  it('displays metrics correctly', () => {
    const store = createTestStore()
    render(
      <Provider store={store}>
        <AgentCard agent={mockAgent} onClick={() => {}} />
      </Provider>
    )
    expect(screen.getByText('99.5%')).toBeInTheDocument() // uptime
    expect(screen.getByText('95.2%')).toBeInTheDocument() // success rate
    expect(screen.getByText('150ms')).toBeInTheDocument() // response time
    expect(screen.getByText('42')).toBeInTheDocument() // tasks
  })
})

describe('MessageCard Component', () => {
  const mockAgentMessage: ChatMessage = {
    id: 'msg-1',
    content: 'Analysis complete. Found 3 anomalies.',
    senderType: 'agent',
    senderName: 'Analytics Bot',
    senderPersonaType: 'analyst',
    senderIcon: '📊',
    timestamp: new Date('2024-02-15T10:30:00'),
  }

  const mockUserMessage: ChatMessage = {
    id: 'msg-2',
    content: 'Show me the details.',
    senderType: 'user',
    senderName: 'User',
    timestamp: new Date('2024-02-15T10:31:00'),
  }

  it('renders agent message with persona colors', () => {
    const store = createTestStore()
    render(
      <Provider store={store}>
        <MessageCard message={mockAgentMessage} />
      </Provider>
    )
    expect(screen.getByText('Analytics Bot')).toBeInTheDocument()
    expect(screen.getByText('Analysis complete. Found 3 anomalies.')).toBeInTheDocument()
  })

  it('displays agent avatar for agent messages', () => {
    const store = createTestStore()
    render(
      <Provider store={store}>
        <MessageCard message={mockAgentMessage} />
      </Provider>
    )
    expect(screen.getByRole('img')).toHaveTextContent('📊')
  })

  it('displays user icon for user messages', () => {
    const store = createTestStore()
    render(
      <Provider store={store}>
        <MessageCard message={mockUserMessage} />
      </Provider>
    )
    expect(screen.getByText('👤')).toBeInTheDocument()
  })

  it('applies bold font for coordinator agent messages', () => {
    const store = createTestStore()
    const coordinatorMessage = { ...mockAgentMessage, senderPersonaType: 'coordinator' as const }
    const { container } = render(
      <Provider store={store}>
        <MessageCard message={coordinatorMessage} />
      </Provider>
    )
    const nameElement = screen.getByText('Analytics Bot')
    expect(nameElement).toHaveClass('font-bold')
  })

  it('applies regular font for user messages', () => {
    const store = createTestStore()
    const { container } = render(
      <Provider store={store}>
        <MessageCard message={mockUserMessage} />
      </Provider>
    )
    const nameElement = screen.getByText('User')
    expect(nameElement).toHaveClass('font-normal')
  })

  it('displays timestamp in correct format', () => {
    const store = createTestStore()
    render(
      <Provider store={store}>
        <MessageCard message={mockAgentMessage} />
      </Provider>
    )
    // Timestamp format: "10:30 AM" or similar (locale-dependent)
    expect(screen.getByText(/\d{1,2}:\d{2}/)).toBeInTheDocument()
  })
})
