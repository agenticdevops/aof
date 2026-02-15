/**
 * WebSocket Redux middleware for real-time event handling
 *
 * This middleware processes WebSocket events and dispatches appropriate Redux actions
 * to update the UI state. It handles deduplication, toast notifications, and animations.
 */

import type { Middleware } from '@reduxjs/toolkit'
import type { RootState } from '@/store/store'
import type { WSEvent } from '@/types/events'
import {
  isHeartbeatEvent,
  isStandupEvent,
  isMessageEvent,
  isAgentStatusChangeEvent,
  isAgentJoinEvent,
  isAgentLeaveEvent,
  isTypingIndicatorEvent,
} from '@/types/events'
import { updateAgent } from '@/store/slices/dashboardSlice'
import {
  addMessage,
  addSquadMember as addChatSquadMember,
  removeSquadMember as removeChatSquadMember,
  setTypingAgent,
  updateMemberStatus,
} from '@/store/slices/chatSlice'
import type { Message, SquadMember } from '@/types/chat'

/**
 * Deduplication cache for WebSocket events
 * Keeps track of recent event IDs to prevent duplicate processing
 */
class EventDeduplicator {
  private recentEventIds = new Set<string>()
  private readonly maxSize = 1000
  private readonly ttlMs = 60000 // 1 minute

  /**
   * Check if an event ID has been seen recently
   */
  isDuplicate(eventId: string): boolean {
    if (this.recentEventIds.has(eventId)) {
      return true
    }

    this.recentEventIds.add(eventId)

    // Clean up old entries if size limit reached
    if (this.recentEventIds.size > this.maxSize) {
      const toDelete = Array.from(this.recentEventIds).slice(0, 100)
      toDelete.forEach((id) => this.recentEventIds.delete(id))
    }

    // Auto-remove after TTL
    setTimeout(() => {
      this.recentEventIds.delete(eventId)
    }, this.ttlMs)

    return false
  }
}

const deduplicator = new EventDeduplicator()

/**
 * Global toast callback (set by ToastProvider)
 */
let toastCallback: ((message: string, type: 'success' | 'error' | 'info' | 'warning') => void) | null = null

export function setToastCallback(
  callback: (message: string, type: 'success' | 'error' | 'info' | 'warning') => void
) {
  toastCallback = callback
}

function showToast(message: string, type: 'success' | 'error' | 'info' | 'warning') {
  if (toastCallback) {
    toastCallback(message, type)
  }
}

/**
 * WebSocket middleware action type
 */
export const WS_EVENT_RECEIVED = 'websocket/eventReceived' as const

interface WebSocketEventAction {
  type: typeof WS_EVENT_RECEIVED
  payload: {
    event: WSEvent
    id?: string
  }
}

/**
 * Action creator for WebSocket events
 */
export function wsEventReceived(event: WSEvent, id?: string): WebSocketEventAction {
  return {
    type: WS_EVENT_RECEIVED,
    payload: { event, id: id || `${event.type}-${Date.now()}` },
  }
}

/**
 * WebSocket middleware
 *
 * Intercepts WS_EVENT_RECEIVED actions and dispatches appropriate Redux actions
 * based on the event type. Also handles toast notifications and deduplication.
 */
export const websocketMiddleware: Middleware<{}, RootState> = (store) => (next) => (action) => {
  // Pass all non-WebSocket actions through
  if (action.type !== WS_EVENT_RECEIVED) {
    return next(action)
  }

  const { event, id } = (action as WebSocketEventAction).payload

  // Deduplication check
  if (deduplicator.isDuplicate(id)) {
    console.debug(`Duplicate event ignored: ${id}`)
    return next(action)
  }

  // Handle different event types
  try {
    if (isHeartbeatEvent(event)) {
      handleHeartbeatEvent(event, store.dispatch)
    } else if (isStandupEvent(event)) {
      handleStandupEvent(event, store.dispatch)
    } else if (isMessageEvent(event)) {
      handleMessageEvent(event, store.dispatch)
    } else if (isAgentStatusChangeEvent(event)) {
      handleAgentStatusChangeEvent(event, store.dispatch)
    } else if (isAgentJoinEvent(event)) {
      handleAgentJoinEvent(event, store.dispatch)
    } else if (isAgentLeaveEvent(event)) {
      handleAgentLeaveEvent(event, store.dispatch)
    } else if (isTypingIndicatorEvent(event)) {
      handleTypingIndicatorEvent(event, store.dispatch)
    } else {
      console.warn('Unknown WebSocket event type:', event)
    }
  } catch (err) {
    console.error('Error handling WebSocket event:', err)
  }

  return next(action)
}

/**
 * Handle HeartbeatEvent - update agent metrics in dashboard
 */
function handleHeartbeatEvent(event: typeof isHeartbeatEvent extends (e: any) => e is infer T ? T : never, dispatch: any) {
  const { agent } = event

  // Update agent in dashboard
  dispatch(
    updateAgent({
      id: agent.id,
      name: agent.id, // Will be updated with full agent data later
      role: 'Agent',
      status: agent.status,
      metrics: agent.metrics,
      personaColor: '#3b82f6', // Default blue
      personaIcon: '🤖',
      updatedAt: new Date(event.timestamp),
    })
  )

  // Update online status in chat
  dispatch(
    updateMemberStatus({
      id: agent.id,
      isOnline: agent.status === 'active',
    })
  )
}

/**
 * Handle StandupEvent - add standup message to chat feed
 */
function handleStandupEvent(event: typeof isStandupEvent extends (e: any) => e is infer T ? T : never, dispatch: any) {
  const { agent, content, timestamp } = event

  const message: Message = {
    id: `standup-${agent.id}-${timestamp}`,
    content,
    type: 'agent',
    sender: {
      id: agent.id,
      name: agent.name,
      role: 'Agent',
      isOnline: true,
      isAgent: true,
      personaColor: '#3b82f6',
      personaIcon: '📊',
    },
    timestamp: new Date(timestamp),
    personaColor: '#3b82f6',
    personaIcon: '📊',
    isRead: false,
  }

  dispatch(addMessage(message))
  showToast(`${agent.name} submitted standup report`, 'info')
}

/**
 * Handle MessageEvent - add message to chat feed
 */
function handleMessageEvent(event: typeof isMessageEvent extends (e: any) => e is infer T ? T : never, dispatch: any) {
  const { sender, content, timestamp } = event

  const message: Message = {
    id: `msg-${sender.id}-${timestamp}`,
    content,
    type: 'agent',
    sender: {
      id: sender.id,
      name: sender.name,
      role: 'Agent',
      isOnline: true,
      isAgent: true,
      personaColor: sender.personaColor,
      personaIcon: sender.personaIcon,
    },
    timestamp: new Date(timestamp),
    personaColor: sender.personaColor,
    personaIcon: sender.personaIcon,
    isRead: false,
  }

  dispatch(addMessage(message))

  // Show toast only if not currently on squad chat page
  // (In a real app, we'd check current route)
  showToast(`New message from ${sender.name}`, 'info')
}

/**
 * Handle AgentStatusChangeEvent - update agent status with animation
 */
function handleAgentStatusChangeEvent(event: typeof isAgentStatusChangeEvent extends (e: any) => e is infer T ? T : never, dispatch: any) {
  const { agent, timestamp } = event

  dispatch(
    updateAgent({
      id: agent.id,
      name: agent.name,
      role: 'Agent',
      status: agent.status,
      metrics: {
        uptime: 0,
        successRate: 0,
        responseTime: 0,
        tasksCompleted: 0,
      },
      personaColor: '#3b82f6',
      personaIcon: '🤖',
      updatedAt: new Date(timestamp),
    })
  )

  dispatch(
    updateMemberStatus({
      id: agent.id,
      isOnline: agent.status === 'active',
    })
  )
}

/**
 * Handle AgentJoinEvent - add agent to squad members
 */
function handleAgentJoinEvent(event: typeof isAgentJoinEvent extends (e: any) => e is infer T ? T : never, dispatch: any) {
  const { agent } = event

  const member: SquadMember = {
    id: agent.id,
    name: agent.name,
    role: agent.role,
    isOnline: true,
    isAgent: true,
    personaColor: agent.personaColor,
    personaIcon: agent.personaIcon,
  }

  dispatch(addChatSquadMember(member))
  showToast(`${agent.name} joined the squad`, 'success')
}

/**
 * Handle AgentLeaveEvent - remove agent from squad members
 */
function handleAgentLeaveEvent(event: typeof isAgentLeaveEvent extends (e: any) => e is infer T ? T : never, dispatch: any) {
  const { agent } = event

  dispatch(removeChatSquadMember(agent.id))
  showToast(`${agent.name} left the squad`, 'warning')
}

/**
 * Handle TypingIndicatorEvent - show typing indicator
 */
function handleTypingIndicatorEvent(event: typeof isTypingIndicatorEvent extends (e: any) => e is infer T ? T : never, dispatch: any) {
  const { agent } = event

  dispatch(setTypingAgent(agent.id))

  // Auto-clear typing indicator after 5 seconds
  setTimeout(() => {
    dispatch(setTypingAgent(null))
  }, 5000)
}
