/**
 * WebSocket client with offline queue, event parsing, and subscription management
 *
 * Features:
 * - Singleton pattern for single WebSocket connection
 * - Event-based subscription system
 * - Offline message queueing with retry
 * - Message deduplication
 * - Automatic reconnection with exponential backoff
 */

import type { WSEvent, WSMessage } from '@/types/events'

/**
 * WebSocket connection status
 */
export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error'

/**
 * Event callback type
 */
type EventCallback = (event: WSEvent) => void

/**
 * Queued message for offline handling
 */
interface QueuedMessage {
  id: string
  message: object
  timestamp: number
  retries: number
}

/**
 * WebSocket client class with enhanced offline support
 */
export class WebSocketClient {
  private static instance: WebSocketClient | null = null
  private ws: WebSocket | null = null
  private url: string
  private reconnectAttempts = 0
  private maxReconnectAttempts = 20
  private reconnectDelay = 1000
  private status: ConnectionStatus = 'disconnected'

  // Subscription management
  private subscriptions: Map<string, Set<EventCallback>> = new Map()
  private globalCallbacks: Set<EventCallback> = new Set()

  // Offline message queue
  private messageQueue: QueuedMessage[] = []
  private readonly maxQueueSize = 100
  private readonly maxRetries = 3

  // Deduplication tracking
  private processedMessageIds = new Set<string>()

  // Reconnection handler
  private reconnectTimeout: number | null = null

  private constructor(url: string) {
    this.url = url
  }

  /**
   * Get singleton instance
   */
  static getInstance(url?: string): WebSocketClient {
    if (!WebSocketClient.instance) {
      const wsUrl = url || import.meta.env.VITE_WS_URL || 'ws://localhost:7777/ws'
      WebSocketClient.instance = new WebSocketClient(wsUrl)
    }
    return WebSocketClient.instance
  }

  /**
   * Connect to WebSocket server
   */
  async connect(): Promise<void> {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return Promise.resolve()
    }

    this.status = 'connecting'

    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.url)

        this.ws.onopen = () => {
          console.log('WebSocket connected')
          this.status = 'connected'
          this.reconnectAttempts = 0
          this.flushQueue()
          resolve()
        }

        this.ws.onerror = (error) => {
          console.error('WebSocket error:', error)
          this.status = 'error'
          reject(error)
        }

        this.ws.onclose = () => {
          console.log('WebSocket disconnected')
          this.status = 'disconnected'
          this.attemptReconnect()
        }

        this.ws.onmessage = (event) => {
          this.handleMessage(event.data)
        }
      } catch (error) {
        this.status = 'error'
        reject(error)
      }
    })
  }

  /**
   * Disconnect from WebSocket server
   */
  disconnect(): void {
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout)
      this.reconnectTimeout = null
    }

    if (this.ws) {
      this.ws.close()
      this.ws = null
    }

    this.status = 'disconnected'
  }

  /**
   * Check if currently connected
   */
  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN
  }

  /**
   * Get current connection status
   */
  getConnectionStatus(): ConnectionStatus {
    return this.status
  }

  /**
   * Subscribe to specific event type
   */
  subscribe(eventType: string, callback: EventCallback): void {
    if (!this.subscriptions.has(eventType)) {
      this.subscriptions.set(eventType, new Set())
    }

    this.subscriptions.get(eventType)!.add(callback)
  }

  /**
   * Subscribe to all events
   */
  subscribeAll(callback: EventCallback): void {
    this.globalCallbacks.add(callback)
  }

  /**
   * Unsubscribe from specific event type
   */
  unsubscribe(eventType: string, callback: EventCallback): void {
    const callbacks = this.subscriptions.get(eventType)
    if (callbacks) {
      callbacks.delete(callback)
      if (callbacks.size === 0) {
        this.subscriptions.delete(eventType)
      }
    }
  }

  /**
   * Unsubscribe from all events
   */
  unsubscribeAll(callback: EventCallback): void {
    this.globalCallbacks.delete(callback)
  }

  /**
   * Send message to server
   * Queues message if offline
   */
  send(message: object): void {
    const messageId = `msg-${Date.now()}-${Math.random().toString(36).substring(7)}`

    if (this.isConnected()) {
      this.sendImmediate(message)
    } else {
      this.queueMessage(messageId, message)
    }
  }

  /**
   * Send message immediately (internal)
   */
  private sendImmediate(message: object): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      try {
        this.ws.send(JSON.stringify(message))
      } catch (error) {
        console.error('Failed to send WebSocket message:', error)
        // Re-queue on send failure
        const messageId = `msg-${Date.now()}-${Math.random().toString(36).substring(7)}`
        this.queueMessage(messageId, message)
      }
    }
  }

  /**
   * Queue message for later sending
   */
  private queueMessage(id: string, message: object): void {
    if (this.messageQueue.length >= this.maxQueueSize) {
      console.warn('Message queue full, dropping oldest message')
      this.messageQueue.shift()
    }

    this.messageQueue.push({
      id,
      message,
      timestamp: Date.now(),
      retries: 0,
    })
  }

  /**
   * Flush queued messages when reconnected
   */
  private flushQueue(): void {
    if (this.messageQueue.length === 0) {
      return
    }

    console.log(`Flushing ${this.messageQueue.length} queued messages`)

    const messages = [...this.messageQueue]
    this.messageQueue = []

    for (const queuedMsg of messages) {
      if (queuedMsg.retries < this.maxRetries) {
        this.sendImmediate(queuedMsg.message)
      } else {
        console.warn(`Message ${queuedMsg.id} exceeded max retries, dropping`)
      }
    }
  }

  /**
   * Handle incoming WebSocket message
   */
  private handleMessage(data: string): void {
    try {
      const parsed = JSON.parse(data)

      // Check for WSMessage wrapper with ID
      if (parsed.event && parsed.id) {
        const wsMessage = parsed as WSMessage

        // Deduplication check
        if (this.processedMessageIds.has(wsMessage.id)) {
          console.debug(`Duplicate message ignored: ${wsMessage.id}`)
          return
        }

        this.processedMessageIds.add(wsMessage.id)

        // Clean up old IDs (keep last 1000)
        if (this.processedMessageIds.size > 1000) {
          const toDelete = Array.from(this.processedMessageIds).slice(0, 100)
          toDelete.forEach(id => this.processedMessageIds.delete(id))
        }

        this.emitEvent(wsMessage.event)
      } else if (parsed.type) {
        // Direct event without wrapper
        this.emitEvent(parsed as WSEvent)
      } else {
        console.warn('Unknown WebSocket message format:', parsed)
      }
    } catch (error) {
      console.error('Failed to parse WebSocket message:', error, data)
    }
  }

  /**
   * Emit event to subscribers
   */
  private emitEvent(event: WSEvent): void {
    // Emit to type-specific subscribers
    const callbacks = this.subscriptions.get(event.type)
    if (callbacks) {
      callbacks.forEach(callback => {
        try {
          callback(event)
        } catch (error) {
          console.error(`Error in event callback for ${event.type}:`, error)
        }
      })
    }

    // Emit to global subscribers
    this.globalCallbacks.forEach(callback => {
      try {
        callback(event)
      } catch (error) {
        console.error('Error in global event callback:', error)
      }
    })
  }

  /**
   * Attempt reconnection with exponential backoff
   */
  private attemptReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Max reconnection attempts reached')
      this.status = 'error'
      return
    }

    this.reconnectAttempts++
    const delay = Math.min(
      this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1),
      30000
    )

    console.log(`Attempting to reconnect in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`)

    this.reconnectTimeout = window.setTimeout(() => {
      this.connect().catch((error) => {
        console.error('Reconnect failed:', error)
      })
    }, delay)
  }

  /**
   * Get queue size (for testing/debugging)
   */
  getQueueSize(): number {
    return this.messageQueue.length
  }

  /**
   * Clear message queue (for testing)
   */
  clearQueue(): void {
    this.messageQueue = []
  }
}

/**
 * Get singleton WebSocket client instance
 */
export function getWebSocketClient(): WebSocketClient {
  return WebSocketClient.getInstance()
}

/**
 * Connect to WebSocket server (convenience function)
 */
export async function connectWebSocket(url?: string): Promise<WebSocketClient> {
  const client = WebSocketClient.getInstance(url)
  await client.connect()
  return client
}

// Default export for backward compatibility
export const wsClient = WebSocketClient.getInstance()
