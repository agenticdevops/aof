/**
 * Chat domain types for Squad Chat interface
 */

/**
 * Message type classification
 */
export type MessageType = 'user' | 'agent' | 'system' | 'announcement'

/**
 * Squad member representation (agent or human)
 */
export interface SquadMember {
  /** Unique member identifier */
  id: string
  /** Display name */
  name: string
  /** Role description (e.g., "Orchestrator", "Human Operator") */
  role: string
  /** Online status indicator */
  isOnline: boolean
  /** Whether this member is an agent (vs human) */
  isAgent: boolean
  /** Persona color for avatar/styling (hex code) */
  personaColor: string
  /** Persona icon (emoji or symbol) */
  personaIcon: string
}

/**
 * Chat message with sender information and persona styling
 */
export interface Message {
  /** Unique message identifier */
  id: string
  /** Message text content */
  content: string
  /** Message type classification */
  type: MessageType
  /** Sender information */
  sender: SquadMember
  /** Message timestamp */
  timestamp: Date
  /** Persona color for message styling (inherited from sender) */
  personaColor: string
  /** Persona icon (inherited from sender) */
  personaIcon: string
  /** Read status (for future read receipts) */
  isRead: boolean
}

/**
 * Chat Redux state
 */
export interface ChatState {
  /** All messages in chronological order */
  messages: Message[]
  /** All squad members (agents and humans) */
  squadMembers: SquadMember[]
  /** Current search query for filtering messages */
  searchQuery: string
  /** Loading state for async operations */
  isLoading: boolean
  /** Error message (if any) */
  error: string | null
  /** ID of agent currently typing (null if none) */
  typingAgentId: string | null
}

/**
 * Message send payload for creating new messages
 */
export interface SendMessagePayload {
  content: string
  type: MessageType
  senderId: string
}
