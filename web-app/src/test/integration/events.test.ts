/**
 * WebSocket middleware event handling tests
 *
 * Tests that WebSocket events are correctly processed by the middleware
 * and dispatch appropriate Redux actions.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { configureStore } from '@reduxjs/toolkit'
import { websocketMiddleware, wsEventReceived } from '@/middleware/websocketMiddleware'
import dashboardReducer from '@/store/slices/dashboardSlice'
import chatReducer from '@/store/slices/chatSlice'
import type {
  HeartbeatEvent,
  StandupEvent,
  MessageEvent,
  AgentStatusChangeEvent,
  AgentJoinEvent,
  AgentLeaveEvent,
  TypingIndicatorEvent,
} from '@/types/events'

describe('WebSocket Middleware Event Handling', () => {
  let store: ReturnType<typeof configureStore>

  beforeEach(() => {
    store = configureStore({
      reducer: {
        dashboard: dashboardReducer,
        chat: chatReducer,
      },
      middleware: (getDefaultMiddleware) =>
        getDefaultMiddleware().concat(websocketMiddleware),
    })
  })

  describe('HeartbeatEvent', () => {
    it('should update agent metrics in dashboard', () => {
      const event: HeartbeatEvent = {
        type: 'heartbeat',
        agent: {
          id: 'agent-1',
          status: 'active',
          metrics: {
            uptime: 99.5,
            successRate: 95.0,
            responseTime: 150,
            tasksCompleted: 42,
          },
        },
        timestamp: new Date().toISOString(),
      }

      store.dispatch(wsEventReceived(event, 'test-heartbeat-1'))

      const state = store.getState()
      const agent = state.dashboard.agents.find((a) => a.id === 'agent-1')

      expect(agent).toBeDefined()
      expect(agent?.status).toBe('active')
      expect(agent?.metrics.uptime).toBe(99.5)
      expect(agent?.metrics.successRate).toBe(95.0)
      expect(agent?.metrics.tasksCompleted).toBe(42)
    })

    it('should update member online status in chat', () => {
      const event: HeartbeatEvent = {
        type: 'heartbeat',
        agent: {
          id: 'agent-1',
          status: 'active',
          metrics: {
            uptime: 99.5,
            successRate: 95.0,
            responseTime: 150,
            tasksCompleted: 42,
          },
        },
        timestamp: new Date().toISOString(),
      }

      // Pre-add member
      store.dispatch({
        type: 'chat/addSquadMember',
        payload: {
          id: 'agent-1',
          name: 'Test Agent',
          role: 'Tester',
          isOnline: false,
          isAgent: true,
          personaColor: '#3b82f6',
          personaIcon: '🤖',
        },
      })

      store.dispatch(wsEventReceived(event, 'test-heartbeat-2'))

      const state = store.getState()
      const member = state.chat.squadMembers.find((m) => m.id === 'agent-1')

      expect(member?.isOnline).toBe(true)
    })
  })

  describe('StandupEvent', () => {
    it('should add standup message to chat feed', () => {
      const event: StandupEvent = {
        type: 'standup',
        agent: {
          id: 'agent-1',
          name: 'Test Agent',
        },
        content: 'Daily standup: Completed 5 tasks today',
        timestamp: new Date().toISOString(),
      }

      store.dispatch(wsEventReceived(event, 'test-standup-1'))

      const state = store.getState()
      const message = state.chat.messages[0]

      expect(message).toBeDefined()
      expect(message.content).toBe('Daily standup: Completed 5 tasks today')
      expect(message.type).toBe('agent')
      expect(message.sender.id).toBe('agent-1')
    })
  })

  describe('MessageEvent', () => {
    it('should add message to chat feed', () => {
      const event: MessageEvent = {
        type: 'message',
        sender: {
          id: 'agent-1',
          name: 'Test Agent',
          personaColor: '#3b82f6',
          personaIcon: '🤖',
        },
        content: 'Hello from agent',
        timestamp: new Date().toISOString(),
      }

      store.dispatch(wsEventReceived(event, 'test-message-1'))

      const state = store.getState()
      const message = state.chat.messages[0]

      expect(message).toBeDefined()
      expect(message.content).toBe('Hello from agent')
      expect(message.sender.name).toBe('Test Agent')
      expect(message.personaColor).toBe('#3b82f6')
    })
  })

  describe('AgentStatusChangeEvent', () => {
    it('should update agent status with animation', () => {
      // Pre-add agent
      store.dispatch({
        type: 'dashboard/updateAgent',
        payload: {
          id: 'agent-1',
          name: 'Test Agent',
          role: 'Tester',
          status: 'idle',
          metrics: {
            uptime: 0,
            successRate: 0,
            responseTime: 0,
            tasksCompleted: 0,
          },
          personaColor: '#3b82f6',
          personaIcon: '🤖',
          updatedAt: new Date(),
        },
      })

      const event: AgentStatusChangeEvent = {
        type: 'agent_status_change',
        agent: {
          id: 'agent-1',
          name: 'Test Agent',
          status: 'active',
        },
        timestamp: new Date().toISOString(),
      }

      store.dispatch(wsEventReceived(event, 'test-status-change-1'))

      const state = store.getState()
      const agent = state.dashboard.agents.find((a) => a.id === 'agent-1')

      expect(agent?.status).toBe('active')
    })
  })

  describe('AgentJoinEvent', () => {
    it('should add member to squad', () => {
      const event: AgentJoinEvent = {
        type: 'agent_join',
        agent: {
          id: 'agent-2',
          name: 'New Agent',
          role: 'Specialist',
          personaColor: '#10b981',
          personaIcon: '🚀',
        },
        timestamp: new Date().toISOString(),
      }

      store.dispatch(wsEventReceived(event, 'test-join-1'))

      const state = store.getState()
      const member = state.chat.squadMembers.find((m) => m.id === 'agent-2')

      expect(member).toBeDefined()
      expect(member?.name).toBe('New Agent')
      expect(member?.role).toBe('Specialist')
      expect(member?.isOnline).toBe(true)
    })
  })

  describe('AgentLeaveEvent', () => {
    it('should remove member from squad', () => {
      // Pre-add member
      store.dispatch({
        type: 'chat/addSquadMember',
        payload: {
          id: 'agent-3',
          name: 'Leaving Agent',
          role: 'Tester',
          isOnline: true,
          isAgent: true,
          personaColor: '#3b82f6',
          personaIcon: '🤖',
        },
      })

      const event: AgentLeaveEvent = {
        type: 'agent_leave',
        agent: {
          id: 'agent-3',
          name: 'Leaving Agent',
        },
        timestamp: new Date().toISOString(),
      }

      store.dispatch(wsEventReceived(event, 'test-leave-1'))

      const state = store.getState()
      const member = state.chat.squadMembers.find((m) => m.id === 'agent-3')

      expect(member).toBeUndefined()
    })
  })

  describe('TypingIndicatorEvent', () => {
    it('should set typing agent', () => {
      const event: TypingIndicatorEvent = {
        type: 'typing_indicator',
        agent: {
          id: 'agent-1',
          name: 'Test Agent',
        },
        timestamp: new Date().toISOString(),
      }

      store.dispatch(wsEventReceived(event, 'test-typing-1'))

      const state = store.getState()
      expect(state.chat.typingAgentId).toBe('agent-1')
    })

    it('should auto-clear typing indicator after delay', async () => {
      vi.useFakeTimers()

      const event: TypingIndicatorEvent = {
        type: 'typing_indicator',
        agent: {
          id: 'agent-1',
          name: 'Test Agent',
        },
        timestamp: new Date().toISOString(),
      }

      store.dispatch(wsEventReceived(event, 'test-typing-2'))

      // Fast-forward time
      vi.advanceTimersByTime(5000)

      const state = store.getState()
      expect(state.chat.typingAgentId).toBe(null)

      vi.useRealTimers()
    })
  })

  describe('Event Deduplication', () => {
    it('should not process duplicate events', () => {
      const event: MessageEvent = {
        type: 'message',
        sender: {
          id: 'agent-1',
          name: 'Test Agent',
          personaColor: '#3b82f6',
          personaIcon: '🤖',
        },
        content: 'Duplicate test',
        timestamp: new Date().toISOString(),
      }

      // Dispatch same event twice with same ID
      store.dispatch(wsEventReceived(event, 'duplicate-test-1'))
      store.dispatch(wsEventReceived(event, 'duplicate-test-1'))

      const state = store.getState()
      // Should only have 1 message
      expect(state.chat.messages.length).toBe(1)
    })

    it('should process events with different IDs', () => {
      const event1: MessageEvent = {
        type: 'message',
        sender: {
          id: 'agent-1',
          name: 'Test Agent',
          personaColor: '#3b82f6',
          personaIcon: '🤖',
        },
        content: 'Message 1',
        timestamp: new Date().toISOString(),
      }

      const event2: MessageEvent = {
        type: 'message',
        sender: {
          id: 'agent-1',
          name: 'Test Agent',
          personaColor: '#3b82f6',
          personaIcon: '🤖',
        },
        content: 'Message 2',
        timestamp: new Date().toISOString(),
      }

      store.dispatch(wsEventReceived(event1, 'unique-test-1'))
      store.dispatch(wsEventReceived(event2, 'unique-test-2'))

      const state = store.getState()
      expect(state.chat.messages.length).toBe(2)
    })
  })
})
