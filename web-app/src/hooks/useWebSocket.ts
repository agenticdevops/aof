/**
 * WebSocket connection management hook
 *
 * Features:
 * - Auto-connect on mount
 * - Auto-reconnect with exponential backoff (3s, 6s, 12s, 30s max)
 * - Connection status tracking
 * - Max 10 reconnection attempts
 * - Event callbacks for connect/disconnect/message/error
 */

import { useEffect, useRef, useState, useCallback } from 'react'
import type { WSEvent } from '@/types/events'

export type WebSocketStatus = 'connecting' | 'connected' | 'disconnected' | 'error'

export interface UseWebSocketOptions {
  /** WebSocket server URL */
  url: string
  /** Callback when a message is received */
  onMessage?: (event: WSEvent) => void
  /** Callback when connection is established */
  onConnect?: () => void
  /** Callback when connection is closed */
  onDisconnect?: () => void
  /** Callback when an error occurs */
  onError?: (error: Error) => void
  /** Maximum reconnection attempts (default: 10) */
  maxReconnectAttempts?: number
}

export interface UseWebSocketReturn {
  /** Current connection status */
  status: WebSocketStatus
  /** WebSocket instance (null if not connected) */
  ws: WebSocket | null
  /** Manually trigger reconnection */
  reconnect: () => void
  /** Manually disconnect */
  disconnect: () => void
}

/**
 * Custom hook for WebSocket connection management
 *
 * Handles connection lifecycle, automatic reconnection with exponential backoff,
 * and event callbacks for real-time communication.
 *
 * @example
 * ```tsx
 * const { status, ws } = useWebSocket({
 *   url: 'ws://localhost:7777/ws',
 *   onMessage: (event) => dispatch(handleEvent(event)),
 *   onConnect: () => console.log('Connected'),
 *   onDisconnect: () => console.log('Disconnected'),
 * })
 * ```
 */
export function useWebSocket(options: UseWebSocketOptions): UseWebSocketReturn {
  const {
    url,
    onMessage,
    onConnect,
    onDisconnect,
    onError,
    maxReconnectAttempts = 10,
  } = options

  const [status, setStatus] = useState<WebSocketStatus>('disconnected')
  const wsRef = useRef<WebSocket | null>(null)
  const reconnectTimeoutRef = useRef<number | null>(null)
  const reconnectAttemptsRef = useRef(0)
  const isManualDisconnectRef = useRef(false)

  /**
   * Connect to WebSocket server
   */
  const connect = useCallback(() => {
    // Clean up existing connection
    if (wsRef.current) {
      wsRef.current.close()
      wsRef.current = null
    }

    try {
      setStatus('connecting')
      const ws = new WebSocket(url)

      ws.onopen = () => {
        setStatus('connected')
        reconnectAttemptsRef.current = 0
        isManualDisconnectRef.current = false
        onConnect?.()
      }

      ws.onmessage = (e) => {
        try {
          const data = JSON.parse(e.data)
          // Handle both wrapped messages (with id) and raw events
          const event = data.event || data
          onMessage?.(event as WSEvent)
        } catch (err) {
          console.error('Failed to parse WebSocket message:', err)
          onError?.(err instanceof Error ? err : new Error('JSON parse error'))
        }
      }

      ws.onerror = () => {
        setStatus('error')
        onError?.(new Error('WebSocket connection error'))
      }

      ws.onclose = () => {
        setStatus('disconnected')
        wsRef.current = null
        onDisconnect?.()

        // Only auto-reconnect if not manually disconnected
        if (!isManualDisconnectRef.current) {
          scheduleReconnect()
        }
      }

      wsRef.current = ws
    } catch (err) {
      setStatus('error')
      onError?.(err instanceof Error ? err : new Error('WebSocket initialization error'))
      scheduleReconnect()
    }
  }, [url, onMessage, onConnect, onDisconnect, onError])

  /**
   * Schedule reconnection attempt with exponential backoff
   */
  const scheduleReconnect = useCallback(() => {
    // Don't reconnect if we've hit max attempts
    if (reconnectAttemptsRef.current >= maxReconnectAttempts) {
      console.warn(`Max reconnection attempts (${maxReconnectAttempts}) reached`)
      setStatus('error')
      return
    }

    // Clear existing timeout
    if (reconnectTimeoutRef.current !== null) {
      window.clearTimeout(reconnectTimeoutRef.current)
    }

    // Calculate delay with exponential backoff: 3s, 6s, 12s, 24s, 30s (capped)
    const baseDelay = 3000 // 3 seconds
    const delay = Math.min(
      baseDelay * Math.pow(2, reconnectAttemptsRef.current),
      30000 // Max 30 seconds
    )

    console.log(
      `Reconnecting in ${delay / 1000}s (attempt ${reconnectAttemptsRef.current + 1}/${maxReconnectAttempts})`
    )

    reconnectTimeoutRef.current = window.setTimeout(() => {
      reconnectAttemptsRef.current++
      connect()
    }, delay)
  }, [connect, maxReconnectAttempts])

  /**
   * Manually trigger reconnection
   */
  const reconnect = useCallback(() => {
    reconnectAttemptsRef.current = 0
    isManualDisconnectRef.current = false
    connect()
  }, [connect])

  /**
   * Manually disconnect
   */
  const disconnect = useCallback(() => {
    isManualDisconnectRef.current = true
    if (reconnectTimeoutRef.current !== null) {
      window.clearTimeout(reconnectTimeoutRef.current)
      reconnectTimeoutRef.current = null
    }
    if (wsRef.current) {
      wsRef.current.close()
      wsRef.current = null
    }
    setStatus('disconnected')
  }, [])

  // Auto-connect on mount
  useEffect(() => {
    connect()

    // Cleanup on unmount
    return () => {
      isManualDisconnectRef.current = true
      if (reconnectTimeoutRef.current !== null) {
        window.clearTimeout(reconnectTimeoutRef.current)
      }
      if (wsRef.current) {
        wsRef.current.close()
      }
    }
  }, [connect])

  return {
    status,
    ws: wsRef.current,
    reconnect,
    disconnect,
  }
}
