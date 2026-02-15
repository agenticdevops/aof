/**
 * Unit tests for chat components
 */
import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { Provider } from 'react-redux'
import { configureStore } from '@reduxjs/toolkit'
import chatReducer from '@/store/slices/chatSlice'
import appReducer from '@/store/slices/appSlice'
import MessageCard from '@/components/chat/MessageCard'
import MessageFeed from '@/components/chat/MessageFeed'
import MessageInput from '@/components/chat/MessageInput'
import SquadMemberList from '@/components/chat/SquadMemberList'
import type { ChatMessage } from '@/components/chat/MessageCard'
import type { SquadMember } from '@/types/chat'

// Test store factory
const createTestStore = () => {
  return configureStore({
    reducer: {
      chat: chatReducer,
      app: appReducer,
    },
  })
}

// Test wrapper with Redux provider
const Wrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <Provider store={createTestStore()}>{children}</Provider>
)

describe('MessageCard', () => {
  it('renders agent message with persona styling', () => {
    const agentMessage: ChatMessage = {
      id: '1',
      content: 'Hello from agent!',
      senderType: 'agent',
      senderName: 'TestBot',
      senderPersonaType: 'orchestrator',
      senderIcon: '🤖',
      timestamp: new Date(),
    }

    render(
      <Wrapper>
        <MessageCard message={agentMessage} />
      </Wrapper>
    )

    expect(screen.getByText('TestBot')).toBeInTheDocument()
    expect(screen.getByText('Hello from agent!')).toBeInTheDocument()
  })

  it('renders user message with different styling', () => {
    const userMessage: ChatMessage = {
      id: '2',
      content: 'Hello from user!',
      senderType: 'user',
      senderName: 'John',
      timestamp: new Date(),
    }

    render(
      <Wrapper>
        <MessageCard message={userMessage} />
      </Wrapper>
    )

    expect(screen.getByText('John')).toBeInTheDocument()
    expect(screen.getByText('Hello from user!')).toBeInTheDocument()
  })

  it('displays relative timestamp', () => {
    const message: ChatMessage = {
      id: '3',
      content: 'Test message',
      senderType: 'user',
      senderName: 'Test',
      timestamp: new Date(),
    }

    render(
      <Wrapper>
        <MessageCard message={message} />
      </Wrapper>
    )

    // Should show time in HH:MM format
    expect(screen.getByText(/\d{1,2}:\d{2}/)).toBeInTheDocument()
  })
})

describe('MessageFeed', () => {
  it('renders messages in chronological order', () => {
    const messages: ChatMessage[] = [
      {
        id: '1',
        content: 'First message',
        senderType: 'user',
        senderName: 'User',
        timestamp: new Date(Date.now() - 2000),
      },
      {
        id: '2',
        content: 'Second message',
        senderType: 'agent',
        senderName: 'Agent',
        senderPersonaType: 'specialist',
        timestamp: new Date(Date.now() - 1000),
      },
    ]

    render(
      <Wrapper>
        <MessageFeed messages={messages} isLoading={false} />
      </Wrapper>
    )

    expect(screen.getByText('First message')).toBeInTheDocument()
    expect(screen.getByText('Second message')).toBeInTheDocument()
  })

  it('shows empty state when no messages', () => {
    render(
      <Wrapper>
        <MessageFeed messages={[]} isLoading={false} />
      </Wrapper>
    )

    expect(screen.getByText('No messages yet')).toBeInTheDocument()
  })

  it('shows loading spinner when loading', () => {
    render(
      <Wrapper>
        <MessageFeed messages={[]} isLoading={true} />
      </Wrapper>
    )

    expect(screen.getByText('Loading messages...')).toBeInTheDocument()
  })
})

describe('MessageInput', () => {
  it('accepts text input', () => {
    const onSend = vi.fn()

    render(<MessageInput onSend={onSend} />)

    const textarea = screen.getByPlaceholderText('Type a message...')
    fireEvent.change(textarea, { target: { value: 'Test message' } })

    expect(textarea).toHaveValue('Test message')
  })

  it('disables send button when input is empty', () => {
    const onSend = vi.fn()

    render(<MessageInput onSend={onSend} />)

    const sendButton = screen.getByRole('button', { name: /send/i })
    expect(sendButton).toBeDisabled()
  })

  it('enables send button when input has text', () => {
    const onSend = vi.fn()

    render(<MessageInput onSend={onSend} />)

    const textarea = screen.getByPlaceholderText('Type a message...')
    fireEvent.change(textarea, { target: { value: 'Test message' } })

    const sendButton = screen.getByRole('button', { name: /send/i })
    expect(sendButton).not.toBeDisabled()
  })

  it('calls onSend with trimmed text when send clicked', () => {
    const onSend = vi.fn()

    render(<MessageInput onSend={onSend} />)

    const textarea = screen.getByPlaceholderText('Type a message...')
    fireEvent.change(textarea, { target: { value: '  Test message  ' } })

    const sendButton = screen.getByRole('button', { name: /send/i })
    fireEvent.click(sendButton)

    expect(onSend).toHaveBeenCalledWith('Test message')
  })

  it('clears input after sending', () => {
    const onSend = vi.fn()

    render(<MessageInput onSend={onSend} />)

    const textarea = screen.getByPlaceholderText('Type a message...')
    fireEvent.change(textarea, { target: { value: 'Test message' } })

    const sendButton = screen.getByRole('button', { name: /send/i })
    fireEvent.click(sendButton)

    expect(textarea).toHaveValue('')
  })
})

describe('SquadMemberList', () => {
  const mockMembers: SquadMember[] = [
    {
      id: '1',
      name: 'Agent1',
      role: 'Orchestrator',
      isOnline: true,
      isAgent: true,
      personaColor: '#10b981',
      personaIcon: '🤖',
    },
    {
      id: '2',
      name: 'User1',
      role: 'Operator',
      isOnline: false,
      isAgent: false,
      personaColor: '#6366f1',
      personaIcon: '👤',
    },
  ]

  it('renders squad members with names and roles', () => {
    render(<SquadMemberList members={mockMembers} />)

    expect(screen.getByText('Agent1')).toBeInTheDocument()
    expect(screen.getByText('Orchestrator')).toBeInTheDocument()
    expect(screen.getByText('User1')).toBeInTheDocument()
    expect(screen.getByText('Operator')).toBeInTheDocument()
  })

  it('shows agent and human badges correctly', () => {
    render(<SquadMemberList members={mockMembers} />)

    expect(screen.getByText('🤖 Agent')).toBeInTheDocument()
    expect(screen.getByText('👤 Human')).toBeInTheDocument()
  })

  it('displays member count in header', () => {
    render(<SquadMemberList members={mockMembers} />)

    expect(screen.getByText(/1 online • 2 total/)).toBeInTheDocument()
  })

  it('shows empty state when no members', () => {
    render(<SquadMemberList members={[]} />)

    expect(screen.getByText('No members found')).toBeInTheDocument()
  })
})
