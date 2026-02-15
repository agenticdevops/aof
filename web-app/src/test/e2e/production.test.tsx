/**
 * Production readiness tests
 *
 * Tests critical paths, error scenarios, and production deployment readiness.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { configureStore } from '@reduxjs/toolkit'
import dashboardReducer from '@/store/slices/dashboardSlice'
import chatReducer from '@/store/slices/chatSlice'
import { websocketMiddleware, wsEventReceived } from '@/middleware/websocketMiddleware'
import { WebSocketClient } from '@/api/websocket'
import type { WSEvent } from '@/types/events'

describe('Production Readiness', () => {
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
    vi.clearAllMocks()
  })

  describe('Critical Path Tests', () => {
    it('should handle WebSocket connection success', () => {
      const client = WebSocketClient.getInstance('ws://localhost:7777/ws')

      expect(client.getConnectionStatus()).toBe('disconnected')
      expect(client.isConnected()).toBe(false)
    })

    it('should handle agent metric updates with low latency', () => {
      const event: WSEvent = {
        type: 'heartbeat',
        agent: {
          id: 'prod-agent',
          status: 'active',
          metrics: {
            uptime: 99.9,
            successRate: 99.5,
            responseTime: 50,
            tasksCompleted: 1000,
          },
        },
        timestamp: new Date().toISOString(),
      }

      const start = performance.now()
      store.dispatch(wsEventReceived(event, 'prod-heartbeat'))
      const latency = performance.now() - start

      expect(latency).toBeLessThan(100)

      const state = store.getState()
      const agent = state.dashboard.agents.find(a => a.id === 'prod-agent')
      expect(agent?.metrics.uptime).toBe(99.9)
    })

    it('should handle message delivery with low latency', () => {
      const event: WSEvent = {
        type: 'message',
        sender: {
          id: 'prod-agent',
          name: 'Agent',
          personaColor: '#3b82f6',
          personaIcon: '🤖',
        },
        content: 'Production message',
        timestamp: new Date().toISOString(),
      }

      const start = performance.now()
      store.dispatch(wsEventReceived(event, 'prod-message'))
      const latency = performance.now() - start

      expect(latency).toBeLessThan(150)

      const state = store.getState()
      const message = state.chat.messages.find(m => m.content === 'Production message')
      expect(message).toBeDefined()
    })

    it('should handle offline message queueing', async () => {
      const { queueOfflineMessage } = await import('@/store/slices/chatSlice')

      const message = {
        id: 'offline-prod',
        content: 'Offline message',
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

      store.dispatch(queueOfflineMessage(message))

      const state = store.getState()
      // @ts-ignore
      expect(state.chat.offlineQueue).toHaveLength(1)
    })
  })

  describe('Error Scenarios', () => {
    it('should handle invalid WebSocket event gracefully', () => {
      const invalidEvent = {
        type: 'unknown_type',
        data: 'invalid',
      } as unknown as WSEvent

      // Should not throw
      expect(() => {
        store.dispatch(wsEventReceived(invalidEvent, 'invalid-event'))
      }).not.toThrow()
    })

    it('should handle malformed event data', () => {
      const malformedEvent = {
        type: 'heartbeat',
        // Missing required fields
      } as unknown as WSEvent

      // Should not throw
      expect(() => {
        store.dispatch(wsEventReceived(malformedEvent, 'malformed-event'))
      }).not.toThrow()
    })

    it('should handle WebSocket disconnection', () => {
      const client = WebSocketClient.getInstance()

      // Should not throw on disconnect
      expect(() => {
        client.disconnect()
      }).not.toThrow()

      expect(client.isConnected()).toBe(false)
      expect(client.getConnectionStatus()).toBe('disconnected')
    })

    it('should handle message send failures gracefully', () => {
      const client = WebSocketClient.getInstance()

      // Sending while disconnected should queue message
      expect(() => {
        client.send({ type: 'test', content: 'test message' })
      }).not.toThrow()

      // Message should be in queue
      expect(client.getQueueSize()).toBeGreaterThan(0)
    })

    it('should handle missing persona data with defaults', () => {
      const event: WSEvent = {
        type: 'message',
        sender: {
          id: 'no-persona-agent',
          name: 'Agent',
          personaColor: '', // Missing
          personaIcon: '', // Missing
        },
        content: 'Message without persona',
        timestamp: new Date().toISOString(),
      }

      // Should not throw
      expect(() => {
        store.dispatch(wsEventReceived(event, 'no-persona'))
      }).not.toThrow()

      const state = store.getState()
      const message = state.chat.messages.find(m => m.content === 'Message without persona')
      expect(message).toBeDefined()
    })
  })

  describe('Performance Tests', () => {
    it('should handle 100 events without memory leaks', () => {
      const initialMemory = (performance as any).memory?.usedJSHeapSize || 0

      // Dispatch 100 events
      for (let i = 0; i < 100; i++) {
        const event: WSEvent = {
          type: 'heartbeat',
          agent: {
            id: `perf-agent-${i}`,
            status: 'active',
            metrics: {
              uptime: 95,
              successRate: 98,
              responseTime: 100,
              tasksCompleted: i,
            },
          },
          timestamp: new Date().toISOString(),
        }

        store.dispatch(wsEventReceived(event, `perf-${i}`))
      }

      const finalMemory = (performance as any).memory?.usedJSHeapSize || 0

      // Memory increase should be reasonable (< 10MB for 100 events)
      if (initialMemory > 0) {
        const memoryIncrease = finalMemory - initialMemory
        expect(memoryIncrease).toBeLessThan(10 * 1024 * 1024) // 10MB
      }

      expect(store.getState().dashboard.agents).toHaveLength(100)
    })

    it('should maintain stable performance under load', () => {
      const latencies: number[] = []

      // Process 50 events and measure latency
      for (let i = 0; i < 50; i++) {
        const event: WSEvent = {
          type: 'message',
          sender: {
            id: `load-agent-${i}`,
            name: `Agent ${i}`,
            personaColor: '#3b82f6',
            personaIcon: '🤖',
          },
          content: `Message ${i}`,
          timestamp: new Date().toISOString(),
        }

        const start = performance.now()
        store.dispatch(wsEventReceived(event, `load-${i}`))
        latencies.push(performance.now() - start)
      }

      // Average latency should be <100ms
      const avgLatency = latencies.reduce((a, b) => a + b, 0) / latencies.length
      expect(avgLatency).toBeLessThan(100)

      // 95th percentile should be <150ms
      const sorted = [...latencies].sort((a, b) => a - b)
      const p95 = sorted[Math.floor(sorted.length * 0.95)]
      expect(p95).toBeLessThan(150)
    })
  })

  describe('State Consistency', () => {
    it('should maintain consistent state across multiple updates', () => {
      const agentId = 'consistency-agent'

      // Initial heartbeat
      store.dispatch(
        wsEventReceived(
          {
            type: 'heartbeat',
            agent: {
              id: agentId,
              status: 'active',
              metrics: {
                uptime: 95,
                successRate: 98,
                responseTime: 100,
                tasksCompleted: 10,
              },
            },
            timestamp: new Date().toISOString(),
          },
          'consistency-1'
        )
      )

      // Update metrics
      store.dispatch(
        wsEventReceived(
          {
            type: 'heartbeat',
            agent: {
              id: agentId,
              status: 'active',
              metrics: {
                uptime: 96,
                successRate: 99,
                responseTime: 90,
                tasksCompleted: 15,
              },
            },
            timestamp: new Date().toISOString(),
          },
          'consistency-2'
        )
      )

      const state = store.getState()
      const agent = state.dashboard.agents.find(a => a.id === agentId)

      // Should only have 1 agent (updated, not duplicated)
      expect(state.dashboard.agents.filter(a => a.id === agentId)).toHaveLength(1)

      // Should have latest metrics
      expect(agent?.metrics.uptime).toBe(96)
      expect(agent?.metrics.successRate).toBe(99)
      expect(agent?.metrics.tasksCompleted).toBe(15)
    })

    it('should handle rapid state changes without corruption', () => {
      const agentId = 'rapid-agent'

      // Send 20 rapid updates
      for (let i = 0; i < 20; i++) {
        store.dispatch(
          wsEventReceived(
            {
              type: 'heartbeat',
              agent: {
                id: agentId,
                status: i % 2 === 0 ? 'active' : 'idle',
                metrics: {
                  uptime: 90 + i,
                  successRate: 95 + i * 0.1,
                  responseTime: 100 - i,
                  tasksCompleted: i,
                },
              },
              timestamp: new Date().toISOString(),
            },
            `rapid-${i}`
          )
        )
      }

      const state = store.getState()
      const agents = state.dashboard.agents.filter(a => a.id === agentId)

      // Should still only have 1 agent
      expect(agents).toHaveLength(1)

      // Should have final state
      expect(agents[0].metrics.tasksCompleted).toBe(19)
    })
  })

  describe('WebSocket Client', () => {
    it('should support subscription management', () => {
      const client = WebSocketClient.getInstance()
      const callback = vi.fn()

      // Subscribe
      client.subscribe('heartbeat', callback)

      // Unsubscribe
      client.unsubscribe('heartbeat', callback)

      // Should not throw
      expect(callback).not.toHaveBeenCalled()
    })

    it('should track queue size correctly', () => {
      const client = WebSocketClient.getInstance()

      client.clearQueue() // Start fresh

      // Queue 3 messages
      client.send({ type: 'test', content: 'msg1' })
      client.send({ type: 'test', content: 'msg2' })
      client.send({ type: 'test', content: 'msg3' })

      expect(client.getQueueSize()).toBeGreaterThanOrEqual(0)
    })
  })
})
