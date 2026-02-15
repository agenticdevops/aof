/**
 * Offline message queueing and retry tests
 *
 * Tests offline message handling, queue management, and reconnection retry logic.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { Provider } from 'react-redux'
import { configureStore } from '@reduxjs/toolkit'
import chatReducer, {
  queueOfflineMessage,
  flushOfflineQueue,
  setMessageStatus,
  selectOfflineQueue,
} from '@/store/slices/chatSlice'
import type { Message } from '@/types/chat'

// Mock WebSocket
const mockWebSocket = {
  send: vi.fn(),
  close: vi.fn(),
  readyState: WebSocket.OPEN,
}

describe('Offline Message Queueing', () => {
  let store: ReturnType<typeof configureStore>

  beforeEach(() => {
    store = configureStore({
      reducer: {
        chat: chatReducer,
      },
    })
    vi.clearAllMocks()
  })

  it('should queue message when offline', () => {
    const message: Message = {
      id: 'msg-1',
      content: 'Hello while offline',
      type: 'user',
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

    act(() => {
      store.dispatch(queueOfflineMessage(message))
    })

    const state = store.getState()
    const queue = selectOfflineQueue(state)

    expect(queue).toHaveLength(1)
    expect(queue[0].id).toBe('msg-1')
    expect(queue[0].status).toBe('pending')
  })

  it('should queue multiple offline messages', () => {
    const messages: Message[] = [1, 2, 3, 4, 5].map(i => ({
      id: `msg-${i}`,
      content: `Message ${i}`,
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
    }))

    act(() => {
      messages.forEach(msg => {
        store.dispatch(queueOfflineMessage(msg))
      })
    })

    const state = store.getState()
    const queue = selectOfflineQueue(state)

    expect(queue).toHaveLength(5)
    queue.forEach((msg, idx) => {
      expect(msg.id).toBe(`msg-${idx + 1}`)
      expect(msg.status).toBe('pending')
    })
  })

  it('should flush queue and mark messages as sent on reconnect', () => {
    // Queue 3 messages
    const messages: Message[] = [1, 2, 3].map(i => ({
      id: `msg-${i}`,
      content: `Message ${i}`,
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
    }))

    act(() => {
      messages.forEach(msg => store.dispatch(queueOfflineMessage(msg)))
    })

    let state = store.getState()
    let queue = selectOfflineQueue(state)
    expect(queue).toHaveLength(3)

    // Flush queue (simulates reconnection)
    act(() => {
      store.dispatch(flushOfflineQueue())
    })

    state = store.getState()
    queue = selectOfflineQueue(state)

    // Queue should be empty
    expect(queue).toHaveLength(0)

    // Messages should be marked as sent
    const chatMessages = state.chat.messages
    expect(chatMessages).toHaveLength(3)
    chatMessages.forEach(msg => {
      expect(msg.status).toBe('sent')
    })
  })

  it('should update message status from pending to sent to received to read', () => {
    const message: Message = {
      id: 'msg-1',
      content: 'Test message',
      type: 'user',
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

    // Queue message (pending)
    act(() => {
      store.dispatch(queueOfflineMessage(message))
    })

    let state = store.getState()
    let chatMessage = state.chat.messages.find(m => m.id === 'msg-1')
    expect(chatMessage?.status).toBe('pending')

    // Flush queue (sent)
    act(() => {
      store.dispatch(flushOfflineQueue())
    })

    state = store.getState()
    chatMessage = state.chat.messages.find(m => m.id === 'msg-1')
    expect(chatMessage?.status).toBe('sent')

    // Backend acknowledges receipt
    act(() => {
      store.dispatch(setMessageStatus({ messageId: 'msg-1', status: 'received' }))
    })

    state = store.getState()
    chatMessage = state.chat.messages.find(m => m.id === 'msg-1')
    expect(chatMessage?.status).toBe('received')

    // Agent reads message
    act(() => {
      store.dispatch(setMessageStatus({ messageId: 'msg-1', status: 'read' }))
    })

    state = store.getState()
    chatMessage = state.chat.messages.find(m => m.id === 'msg-1')
    expect(chatMessage?.status).toBe('read')
  })

  it('should maintain message order after queue flush', () => {
    const messages: Message[] = [1, 2, 3, 4, 5].map(i => ({
      id: `msg-${i}`,
      content: `Message ${i}`,
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
      timestamp: new Date(Date.now() + i * 1000), // 1 second apart
      personaColor: '#3b82f6',
      personaIcon: '👤',
      isRead: false,
    }))

    act(() => {
      messages.forEach(msg => store.dispatch(queueOfflineMessage(msg)))
    })

    act(() => {
      store.dispatch(flushOfflineQueue())
    })

    const state = store.getState()
    const chatMessages = state.chat.messages

    // Messages should be in chronological order
    expect(chatMessages).toHaveLength(5)
    chatMessages.forEach((msg, idx) => {
      expect(msg.id).toBe(`msg-${idx + 1}`)
    })
  })

  it('should handle empty queue flush gracefully', () => {
    act(() => {
      store.dispatch(flushOfflineQueue())
    })

    const state = store.getState()
    const queue = selectOfflineQueue(state)

    expect(queue).toHaveLength(0)
    expect(state.chat.messages).toHaveLength(0)
  })
})

describe('Message Status Flow', () => {
  let store: ReturnType<typeof configureStore>

  beforeEach(() => {
    store = configureStore({
      reducer: {
        chat: chatReducer,
      },
    })
  })

  it('should track delivery status progression', () => {
    const message: Message = {
      id: 'msg-status-test',
      content: 'Status test message',
      type: 'user',
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
      status: 'pending',
    }

    // Start as pending
    act(() => {
      store.dispatch(queueOfflineMessage(message))
    })

    // Progress through states
    const statuses: Array<'pending' | 'sent' | 'received' | 'read'> = ['sent', 'received', 'read']

    statuses.forEach(status => {
      act(() => {
        if (status === 'sent') {
          store.dispatch(flushOfflineQueue())
        } else {
          store.dispatch(setMessageStatus({ messageId: 'msg-status-test', status }))
        }
      })

      const state = store.getState()
      const msg = state.chat.messages.find(m => m.id === 'msg-status-test')
      expect(msg?.status).toBe(status)
    })
  })
})
