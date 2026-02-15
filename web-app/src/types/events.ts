/**
 * WebSocket event type definitions for real-time communication
 *
 * These events are received from the backend via WebSocket connection
 * and trigger UI updates, animations, and notifications in the frontend.
 */

import type { AgentMetrics, AgentStatus } from './dashboard'

/**
 * Agent heartbeat event - periodic status and metrics update
 */
export interface HeartbeatEvent {
  type: 'heartbeat'
  agent: {
    id: string
    status: AgentStatus
    metrics: AgentMetrics
  }
  timestamp: string
}

/**
 * Standup report event - agent daily/periodic standup message
 */
export interface StandupEvent {
  type: 'standup'
  agent: {
    id: string
    name: string
  }
  content: string
  timestamp: string
}

/**
 * Chat message event - real-time message from agent or user
 */
export interface MessageEvent {
  type: 'message'
  sender: {
    id: string
    name: string
    personaColor: string
    personaIcon: string
  }
  content: string
  timestamp: string
}

/**
 * Agent status change event - agent transitions between active/idle/error
 */
export interface AgentStatusChangeEvent {
  type: 'agent_status_change'
  agent: {
    id: string
    name: string
    status: AgentStatus
  }
  timestamp: string
}

/**
 * Agent join event - new agent joins the squad
 */
export interface AgentJoinEvent {
  type: 'agent_join'
  agent: {
    id: string
    name: string
    role: string
    personaColor: string
    personaIcon: string
  }
  timestamp: string
}

/**
 * Agent leave event - agent disconnects from the squad
 */
export interface AgentLeaveEvent {
  type: 'agent_leave'
  agent: {
    id: string
    name: string
  }
  timestamp: string
}

/**
 * Typing indicator event - shows when agent is composing a message
 */
export interface TypingIndicatorEvent {
  type: 'typing_indicator'
  agent: {
    id: string
    name: string
  }
  timestamp: string
}

/**
 * Discriminated union of all WebSocket event types
 *
 * TypeScript will narrow the type based on the 'type' field,
 * enabling type-safe event handling.
 */
export type WSEvent =
  | HeartbeatEvent
  | StandupEvent
  | MessageEvent
  | AgentStatusChangeEvent
  | AgentJoinEvent
  | AgentLeaveEvent
  | TypingIndicatorEvent

/**
 * WebSocket message wrapper with deduplication ID
 *
 * The id field is used to prevent duplicate event handling
 * if the same event is received multiple times.
 */
export interface WSMessage {
  /** The actual event payload */
  event: WSEvent
  /** Unique message ID for deduplication */
  id: string
}

/**
 * Type guard to check if an event is a HeartbeatEvent
 */
export function isHeartbeatEvent(event: WSEvent): event is HeartbeatEvent {
  return event.type === 'heartbeat'
}

/**
 * Type guard to check if an event is a StandupEvent
 */
export function isStandupEvent(event: WSEvent): event is StandupEvent {
  return event.type === 'standup'
}

/**
 * Type guard to check if an event is a MessageEvent
 */
export function isMessageEvent(event: WSEvent): event is MessageEvent {
  return event.type === 'message'
}

/**
 * Type guard to check if an event is an AgentStatusChangeEvent
 */
export function isAgentStatusChangeEvent(event: WSEvent): event is AgentStatusChangeEvent {
  return event.type === 'agent_status_change'
}

/**
 * Type guard to check if an event is an AgentJoinEvent
 */
export function isAgentJoinEvent(event: WSEvent): event is AgentJoinEvent {
  return event.type === 'agent_join'
}

/**
 * Type guard to check if an event is an AgentLeaveEvent
 */
export function isAgentLeaveEvent(event: WSEvent): event is AgentLeaveEvent {
  return event.type === 'agent_leave'
}

/**
 * Type guard to check if an event is a TypingIndicatorEvent
 */
export function isTypingIndicatorEvent(event: WSEvent): event is TypingIndicatorEvent {
  return event.type === 'typing_indicator'
}
