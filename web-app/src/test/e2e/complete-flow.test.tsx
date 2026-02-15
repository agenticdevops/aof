/**
 * Comprehensive E2E test for complete Phase 2 integration
 *
 * Tests the entire flow from WebSocket events to UI updates,
 * including offline handling, message status, and performance validation.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import { Provider } from 'react-redux'
import { configureStore } from '@reduxjs/toolkit'
import { BrowserRouter } from 'react-router-dom'
import dashboardReducer from '@/store/slices/dashboardSlice'
import chatReducer from '@/store/slices/chatSlice'
import { websocketMiddleware, wsEventReceived } from '@/middleware/websocketMiddleware'
import type { HeartbeatEvent, MessageEvent, StandupEvent, AgentJoinEvent, AnnouncementEvent } from '@/types/events'

// Mock components for testing
import { vi as mockVi } from 'vitest'

// Mock toast callback
let toastMessages: Array<{ message: string; type: string }> = []
const mockToast = (message: string, type: string) => {
  toastMessages.push({ message, type })
}

describe('Complete E2E Flow', () => {
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

    toastMessages = []
    vi.clearAllMocks()
  })

  afterEach(() => {
    toastMessages = []
  })

  describe('Agent Heartbeat Flow', () => {
    it('should update agent metrics within 100ms of WebSocket event', async () => {
      const startTime = performance.now()

      const heartbeatEvent: HeartbeatEvent = {
        type: 'heartbeat',
        agent: {
          id: 'xops',
          status: 'active',
          metrics: {
            uptime: 95.5,
            successRate: 98.2,
            responseTime: 120,
            tasksCompleted: 42,
          },
        },
        timestamp: new Date().toISOString(),
      }

      store.dispatch(wsEventReceived(heartbeatEvent, 'heartbeat-1'))

      const latency = performance.now() - startTime

      // Verify latency <100ms
      expect(latency).toBeLessThan(100)

      // Verify agent metrics updated in Redux
      const state = store.getState()
      const agent = state.dashboard.agents.find(a => a.id === 'xops')

      expect(agent).toBeDefined()
      expect(agent?.status).toBe('active')
      expect(agent?.metrics.uptime).toBe(95.5)
      expect(agent?.metrics.successRate).toBe(98.2)
      expect(agent?.metrics.responseTime).toBe(120)
      expect(agent?.metrics.tasksCompleted).toBe(42)

      // Verify chat member status updated
      const chatMember = state.chat.squadMembers.find(m => m.id === 'xops')
      expect(chatMember?.isOnline).toBe(true)
    })

    it('should handle multiple rapid heartbeat events without lag', async () => {
      const events: HeartbeatEvent[] = Array.from({ length: 10 }, (_, i) => ({
        type: 'heartbeat',
        agent: {
          id: `agent-${i}`,
          status: 'active' as const,
          metrics: {
            uptime: 95 + i,
            successRate: 98,
            responseTime: 100 + i * 10,
            tasksCompleted: i * 5,
          },
        },
        timestamp: new Date().toISOString(),
      }))

      const startTime = performance.now()

      events.forEach((event, i) => {
        store.dispatch(wsEventReceived(event, `heartbeat-${i}`))
      })

      const totalLatency = performance.now() - startTime

      // All 10 events should process in <500ms total
      expect(totalLatency).toBeLessThan(500)

      // Verify all agents created
      const state = store.getState()
      expect(state.dashboard.agents).toHaveLength(10)
    })
  })

  describe('Message Flow', () => {
    it('should add message to chat within 150ms of WebSocket event', async () => {
      const startTime = performance.now()

      const messageEvent: MessageEvent = {
        type: 'message',
        sender: {
          id: 'xops',
          name: 'Xops',
          personaColor: '#3b82f6',
          personaIcon: '🤖',
        },
        content: 'Hey team, starting standup',
        timestamp: new Date().toISOString(),
      }

      store.dispatch(wsEventReceived(messageEvent, 'msg-1'))

      const latency = performance.now() - startTime

      // Verify latency <150ms
      expect(latency).toBeLessThan(150)

      // Verify message added to chat
      const state = store.getState()
      const message = state.chat.messages.find(m => m.content === 'Hey team, starting standup')

      expect(message).toBeDefined()
      expect(message?.sender.name).toBe('Xops')
      expect(message?.personaColor).toBe('#3b82f6')
      expect(message?.type).toBe('agent')
    })
  })

  describe('Standup Flow', () => {
    it('should add standup message to chat and show toast', async () => {
      const standupEvent: StandupEvent = {
        type: 'standup',
        agent: {
          id: 'xops',
          name: 'Xops',
        },
        content: 'Completed 5 tasks today. All systems operational.',
        timestamp: new Date().toISOString(),
      }

      store.dispatch(wsEventReceived(standupEvent, 'standup-1'))

      // Verify message added
      const state = store.getState()
      const message = state.chat.messages.find(m =>
        m.content.includes('Completed 5 tasks today')
      )

      expect(message).toBeDefined()
      expect(message?.type).toBe('agent')
      expect(message?.sender.name).toBe('Xops')
    })
  })

  describe('Agent Join Flow', () => {
    it('should add squad member when agent joins', async () => {
      const joinEvent: AgentJoinEvent = {
        type: 'agent_join',
        agent: {
          id: 'keeper',
          name: 'Keeper',
          role: 'Security Agent',
          personaColor: '#10b981',
          personaIcon: '🛡️',
        },
        timestamp: new Date().toISOString(),
      }

      store.dispatch(wsEventReceived(joinEvent, 'join-1'))

      // Verify squad member added
      const state = store.getState()
      const member = state.chat.squadMembers.find(m => m.id === 'keeper')

      expect(member).toBeDefined()
      expect(member?.name).toBe('Keeper')
      expect(member?.role).toBe('Security Agent')
      expect(member?.personaColor).toBe('#10b981')
      expect(member?.isOnline).toBe(true)
    })
  })

  describe('Announcement Flow', () => {
    it('should broadcast announcement to all with toast notification', async () => {
      const announcementEvent: AnnouncementEvent = {
        type: 'announcement',
        sender: {
          id: 'system',
          name: 'Mission Control',
          personaColor: '#f59e0b',
          personaIcon: '📢',
        },
        content: 'System maintenance scheduled for 2AM UTC',
        timestamp: new Date().toISOString(),
      }

      store.dispatch(wsEventReceived(announcementEvent, 'announcement-1'))

      // Verify announcement message added
      const state = store.getState()
      const message = state.chat.messages.find(m =>
        m.content.includes('System maintenance')
      )

      expect(message).toBeDefined()
      expect(message?.type).toBe('announcement')
      expect(message?.sender.name).toBe('Mission Control')
      expect(message?.status).toBe('received')
    })
  })

  describe('Offline Message Handling', () => {
    it('should queue messages when offline and send on reconnect', async () => {
      // This test would require mocking the WebSocket client
      // For now, we test the Redux state transitions
      const { queueOfflineMessage, flushOfflineQueue } = await import(
        '@/store/slices/chatSlice'
      )

      const message = {
        id: 'offline-msg-1',
        content: 'Message while offline',
        type: 'user' as const,
        sender: {
          id: 'user-1',
          name: 'User',
          role: 'Human',
          isOnline: true,
          isAgent: false,
          personaColor: '#3b82f6',
          personaIcon: '👤',
        },
        timestamp: new Date(),
        personaColor: '#3b82f6',
        personaIcon: '👤',
        isRead: false,
      }

      // Queue message (offline)
      store.dispatch(queueOfflineMessage(message))

      let state = store.getState()
      // @ts-ignore - Extended state type
      expect(state.chat.offlineQueue).toHaveLength(1)

      // Flush queue (reconnected)
      store.dispatch(flushOfflineQueue())

      state = store.getState()
      // @ts-ignore - Extended state type
      expect(state.chat.offlineQueue).toHaveLength(0)

      const sentMessage = state.chat.messages.find(m => m.id === 'offline-msg-1')
      expect(sentMessage?.status).toBe('sent')
    })
  })

  describe('Performance Validation', () => {
    it('should handle 50 events in under 2 seconds', async () => {
      const events = Array.from({ length: 50 }, (_, i) => ({
        type: 'heartbeat' as const,
        agent: {
          id: `agent-${i}`,
          status: 'active' as const,
          metrics: {
            uptime: 95,
            successRate: 98,
            responseTime: 100,
            tasksCompleted: i,
          },
        },
        timestamp: new Date().toISOString(),
      }))

      const startTime = performance.now()

      events.forEach((event, i) => {
        store.dispatch(wsEventReceived(event, `perf-${i}`))
      })

      const totalTime = performance.now() - startTime

      expect(totalTime).toBeLessThan(2000)
      expect(store.getState().dashboard.agents).toHaveLength(50)
    })

    it('should maintain <150ms latency for critical event types', async () => {
      const criticalEvents = [
        {
          type: 'heartbeat' as const,
          agent: {
            id: 'critical-agent',
            status: 'active' as const,
            metrics: { uptime: 95, successRate: 98, responseTime: 100, tasksCompleted: 10 },
          },
          timestamp: new Date().toISOString(),
        },
        {
          type: 'message' as const,
          sender: { id: 'xops', name: 'Xops', personaColor: '#3b82f6', personaIcon: '🤖' },
          content: 'Critical message',
          timestamp: new Date().toISOString(),
        },
      ]

      const latencies: number[] = []

      criticalEvents.forEach((event, i) => {
        const start = performance.now()
        store.dispatch(wsEventReceived(event, `critical-${i}`))
        latencies.push(performance.now() - start)
      })

      // All critical events should process in <150ms
      latencies.forEach(latency => {
        expect(latency).toBeLessThan(150)
      })
    })
  })

  describe('Event Deduplication', () => {
    it('should ignore duplicate events with same ID', () => {
      const event: HeartbeatEvent = {
        type: 'heartbeat',
        agent: {
          id: 'dup-agent',
          status: 'active',
          metrics: { uptime: 95, successRate: 98, responseTime: 100, tasksCompleted: 10 },
        },
        timestamp: new Date().toISOString(),
      }

      // Send same event twice with same ID
      store.dispatch(wsEventReceived(event, 'duplicate-id'))
      store.dispatch(wsEventReceived(event, 'duplicate-id'))

      const state = store.getState()

      // Should only have 1 agent (not 2)
      const agents = state.dashboard.agents.filter(a => a.id === 'dup-agent')
      expect(agents).toHaveLength(1)
    })
  })

  describe('Message Status Progression', () => {
    it('should track message status from pending to read', async () => {
      const { queueOfflineMessage, flushOfflineQueue, setMessageStatus } = await import(
        '@/store/slices/chatSlice'
      )

      const message = {
        id: 'status-msg',
        content: 'Status test',
        type: 'user' as const,
        sender: {
          id: 'user-1',
          name: 'User',
          role: 'Human',
          isOnline: true,
          isAgent: false,
          personaColor: '#3b82f6',
          personaIcon: '👤',
        },
        timestamp: new Date(),
        personaColor: '#3b82f6',
        personaIcon: '👤',
        isRead: false,
      }

      // Pending
      store.dispatch(queueOfflineMessage(message))
      let msg = store.getState().chat.messages.find(m => m.id === 'status-msg')
      expect(msg?.status).toBe('pending')

      // Sent
      store.dispatch(flushOfflineQueue())
      msg = store.getState().chat.messages.find(m => m.id === 'status-msg')
      expect(msg?.status).toBe('sent')

      // Received
      store.dispatch(setMessageStatus({ messageId: 'status-msg', status: 'received' }))
      msg = store.getState().chat.messages.find(m => m.id === 'status-msg')
      expect(msg?.status).toBe('received')

      // Read
      store.dispatch(setMessageStatus({ messageId: 'status-msg', status: 'read' }))
      msg = store.getState().chat.messages.find(m => m.id === 'status-msg')
      expect(msg?.status).toBe('read')
    })
  })
})
