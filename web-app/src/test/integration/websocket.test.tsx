/**
 * WebSocket hook integration tests
 *
 * Tests the useWebSocket hook behavior including connection,
 * reconnection, error handling, and callbacks.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { useWebSocket } from '@/hooks/useWebSocket'
import type { WSEvent } from '@/types/events'

// Mock WebSocket
class MockWebSocket {
  static CONNECTING = 0
  static OPEN = 1
  static CLOSING = 2
  static CLOSED = 3

  public onopen: ((event: Event) => void) | null = null
  public onclose: ((event: CloseEvent) => void) | null = null
  public onerror: ((event: Event) => void) | null = null
  public onmessage: ((event: MessageEvent) => void) | null = null
  public readyState = MockWebSocket.CONNECTING
  public url: string

  constructor(url: string) {
    this.url = url
    // Simulate connection after a tick
    setTimeout(() => {
      this.readyState = MockWebSocket.OPEN
      this.onopen?.(new Event('open'))
    }, 0)
  }

  send(data: string) {
    // Mock send
  }

  close() {
    this.readyState = MockWebSocket.CLOSED
    this.onclose?.(new CloseEvent('close'))
  }

  // Helper to simulate receiving a message
  simulateMessage(data: any) {
    const event = new MessageEvent('message', {
      data: JSON.stringify(data),
    })
    this.onmessage?.(event)
  }

  // Helper to simulate error
  simulateError() {
    this.onerror?.(new Event('error'))
  }
}

describe('useWebSocket Hook', () => {
  let originalWebSocket: typeof WebSocket

  beforeEach(() => {
    originalWebSocket = global.WebSocket
    // @ts-expect-error - mocking WebSocket
    global.WebSocket = MockWebSocket
    vi.useFakeTimers()
  })

  afterEach(() => {
    global.WebSocket = originalWebSocket
    vi.useRealTimers()
  })

  describe('Connection Management', () => {
    it('should connect on mount', async () => {
      const onConnect = vi.fn()

      renderHook(() =>
        useWebSocket({
          url: 'ws://localhost:7777/ws',
          onConnect,
        })
      )

      // Fast-forward to allow connection
      await vi.runAllTimersAsync()

      expect(onConnect).toHaveBeenCalledTimes(1)
    })

    it('should call onMessage when message is received', async () => {
      const onMessage = vi.fn()
      let mockWs: MockWebSocket | null = null

      // Intercept WebSocket construction
      const OriginalMock = global.WebSocket
      // @ts-expect-error - mocking
      global.WebSocket = class extends OriginalMock {
        constructor(url: string) {
          super(url)
          mockWs = this as any
        }
      }

      renderHook(() =>
        useWebSocket({
          url: 'ws://localhost:7777/ws',
          onMessage,
        })
      )

      await vi.runAllTimersAsync()

      // Simulate receiving a message
      const event: WSEvent = {
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

      mockWs?.simulateMessage({ event })
      await vi.runAllTimersAsync()

      expect(onMessage).toHaveBeenCalledWith(event)
    })

    it('should update status to connected when connection opens', async () => {
      const { result } = renderHook(() =>
        useWebSocket({
          url: 'ws://localhost:7777/ws',
        })
      )

      // Initial status should be disconnected
      expect(result.current.status).toBe('connecting')

      await vi.runAllTimersAsync()

      // After connection, status should be connected
      await waitFor(() => {
        expect(result.current.status).toBe('connected')
      })
    })

    it('should call onDisconnect when connection closes', async () => {
      const onDisconnect = vi.fn()
      let mockWs: MockWebSocket | null = null

      const OriginalMock = global.WebSocket
      // @ts-expect-error - mocking
      global.WebSocket = class extends OriginalMock {
        constructor(url: string) {
          super(url)
          mockWs = this as any
        }
      }

      const { result } = renderHook(() =>
        useWebSocket({
          url: 'ws://localhost:7777/ws',
          onDisconnect,
        })
      )

      await vi.runAllTimersAsync()

      // Close connection
      mockWs?.close()
      await vi.runAllTimersAsync()

      expect(onDisconnect).toHaveBeenCalled()
      expect(result.current.status).toBe('disconnected')
    })

    it('should call onError when error occurs', async () => {
      const onError = vi.fn()
      let mockWs: MockWebSocket | null = null

      const OriginalMock = global.WebSocket
      // @ts-expect-error - mocking
      global.WebSocket = class extends OriginalMock {
        constructor(url: string) {
          super(url)
          mockWs = this as any
        }
      }

      renderHook(() =>
        useWebSocket({
          url: 'ws://localhost:7777/ws',
          onError,
        })
      )

      await vi.runAllTimersAsync()

      // Simulate error
      mockWs?.simulateError()
      await vi.runAllTimersAsync()

      expect(onError).toHaveBeenCalled()
    })
  })

  describe('Reconnection Logic', () => {
    it('should reconnect with exponential backoff', async () => {
      let constructorCallCount = 0
      const OriginalMock = global.WebSocket

      // @ts-expect-error - mocking
      global.WebSocket = class extends OriginalMock {
        constructor(url: string) {
          super(url)
          constructorCallCount++
          // Immediately close to trigger reconnection
          setTimeout(() => this.close(), 0)
        }
      }

      renderHook(() =>
        useWebSocket({
          url: 'ws://localhost:7777/ws',
          maxReconnectAttempts: 3,
        })
      )

      // Initial connection
      await vi.runAllTimersAsync()
      expect(constructorCallCount).toBe(1)

      // First reconnect (3s delay)
      await vi.advanceTimersByTimeAsync(3000)
      expect(constructorCallCount).toBe(2)

      // Second reconnect (6s delay)
      await vi.advanceTimersByTimeAsync(6000)
      expect(constructorCallCount).toBe(3)

      // Third reconnect (12s delay)
      await vi.advanceTimersByTimeAsync(12000)
      expect(constructorCallCount).toBe(4)
    })

    it('should stop reconnecting after max attempts', async () => {
      let constructorCallCount = 0
      const OriginalMock = global.WebSocket

      // @ts-expect-error - mocking
      global.WebSocket = class extends OriginalMock {
        constructor(url: string) {
          super(url)
          constructorCallCount++
          setTimeout(() => this.close(), 0)
        }
      }

      renderHook(() =>
        useWebSocket({
          url: 'ws://localhost:7777/ws',
          maxReconnectAttempts: 2,
        })
      )

      // Initial + 2 reconnects = 3 total
      await vi.runAllTimersAsync()

      // Should not exceed 3 attempts
      expect(constructorCallCount).toBeLessThanOrEqual(3)
    })

    it('should reset reconnect attempts on successful connection', async () => {
      let shouldFailConnection = true
      let constructorCallCount = 0
      const OriginalMock = global.WebSocket

      // @ts-expect-error - mocking
      global.WebSocket = class extends OriginalMock {
        constructor(url: string) {
          super(url)
          constructorCallCount++
          if (shouldFailConnection) {
            setTimeout(() => this.close(), 0)
          }
        }
      }

      renderHook(() =>
        useWebSocket({
          url: 'ws://localhost:7777/ws',
        })
      )

      // First attempt fails
      await vi.runAllTimersAsync()
      expect(constructorCallCount).toBe(1)

      // Allow next connection to succeed
      shouldFailConnection = false

      // Reconnect
      await vi.advanceTimersByTimeAsync(3000)
      await vi.runAllTimersAsync()

      // Should have connected successfully
      expect(constructorCallCount).toBe(2)
    })
  })

  describe('Manual Controls', () => {
    it('should reconnect manually', async () => {
      let constructorCallCount = 0
      const OriginalMock = global.WebSocket

      // @ts-expect-error - mocking
      global.WebSocket = class extends OriginalMock {
        constructor(url: string) {
          super(url)
          constructorCallCount++
        }
      }

      const { result } = renderHook(() =>
        useWebSocket({
          url: 'ws://localhost:7777/ws',
        })
      )

      await vi.runAllTimersAsync()
      expect(constructorCallCount).toBe(1)

      // Manual reconnect
      result.current.reconnect()
      await vi.runAllTimersAsync()

      expect(constructorCallCount).toBe(2)
    })

    it('should disconnect manually', async () => {
      const onDisconnect = vi.fn()
      let mockWs: MockWebSocket | null = null

      const OriginalMock = global.WebSocket
      // @ts-expect-error - mocking
      global.WebSocket = class extends OriginalMock {
        constructor(url: string) {
          super(url)
          mockWs = this as any
        }
      }

      const { result } = renderHook(() =>
        useWebSocket({
          url: 'ws://localhost:7777/ws',
          onDisconnect,
        })
      )

      await vi.runAllTimersAsync()

      // Manual disconnect
      result.current.disconnect()
      await vi.runAllTimersAsync()

      expect(result.current.status).toBe('disconnected')
    })
  })

  describe('Error Handling', () => {
    it('should handle JSON parse errors gracefully', async () => {
      const onError = vi.fn()
      let mockWs: MockWebSocket | null = null

      const OriginalMock = global.WebSocket
      // @ts-expect-error - mocking
      global.WebSocket = class extends OriginalMock {
        constructor(url: string) {
          super(url)
          mockWs = this as any
        }
      }

      renderHook(() =>
        useWebSocket({
          url: 'ws://localhost:7777/ws',
          onError,
        })
      )

      await vi.runAllTimersAsync()

      // Send invalid JSON
      const event = new MessageEvent('message', {
        data: 'invalid json {{{',
      })
      mockWs?.onmessage?.(event)

      expect(onError).toHaveBeenCalled()
    })
  })
})
