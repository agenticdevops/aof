import React, { useEffect } from 'react'
import { useAppDispatch, useAppSelector } from '@/store/hooks'
import {
  addMessage,
  setMessages,
  setSquadMembers,
  setSearchQuery,
  useFilteredMessages,
  useSquadMembers,
  useSearchQuery,
} from '@/store/slices/chatSlice'
import type { ChatMessage } from '@/components/chat/MessageCard'
import type { SquadMember } from '@/types/chat'
import MessageFeed from '@/components/chat/MessageFeed'
import MessageInput from '@/components/chat/MessageInput'
import SquadMemberList from '@/components/chat/SquadMemberList'
import { Input } from '@/components/common/Input'
import { Search } from 'lucide-react'

/**
 * SquadChat - Team messaging interface for agent communication
 *
 * Features:
 * - Message feed with persona-styled messages
 * - Message input with keyboard shortcuts
 * - Squad member sidebar with online indicators
 * - Search/filter messages
 * - Mock data for development (replaced by WebSocket in Plan 05)
 */
const SquadChat: React.FC = () => {
  const dispatch = useAppDispatch()
  const filteredMessages = useFilteredMessages()
  const squadMembers = useSquadMembers()
  const searchQuery = useSearchQuery()
  const isLoading = useAppSelector((state) => state.chat.isLoading)

  // Load mock data on mount
  // TODO: Replace with WebSocket integration in Plan 05
  useEffect(() => {
    // Mock squad members (agents and humans)
    const mockMembers: SquadMember[] = [
      {
        id: 'user-1',
        name: 'You',
        role: 'Human Operator',
        isOnline: true,
        isAgent: false,
        personaColor: '#6366f1', // indigo-500
        personaIcon: '👤',
      },
      {
        id: 'agent-1',
        name: 'Xops',
        role: 'Orchestrator',
        isOnline: true,
        isAgent: true,
        personaColor: '#10b981', // emerald-500
        personaIcon: '⚙️',
      },
      {
        id: 'agent-2',
        name: 'K8sOps',
        role: 'Kubernetes Specialist',
        isOnline: true,
        isAgent: true,
        personaColor: '#3b82f6', // blue-500
        personaIcon: '☸️',
      },
      {
        id: 'agent-3',
        name: 'SREWatch',
        role: 'Observability Agent',
        isOnline: false,
        isAgent: true,
        personaColor: '#8b5cf6', // violet-500
        personaIcon: '👁️',
      },
      {
        id: 'agent-4',
        name: 'InfraBot',
        role: 'Infrastructure Agent',
        isOnline: true,
        isAgent: true,
        personaColor: '#f59e0b', // amber-500
        personaIcon: '🏗️',
      },
    ]

    // Mock messages (mix of agent and user messages)
    const mockMessages: ChatMessage[] = [
      {
        id: 'msg-1',
        content: 'Squad online and ready! Monitoring systems active.',
        senderType: 'agent',
        senderName: 'Xops',
        senderPersonaType: 'orchestrator',
        senderIcon: '⚙️',
        timestamp: new Date(Date.now() - 3600000), // 1 hour ago
      },
      {
        id: 'msg-2',
        content: 'Kubernetes cluster health check complete. All pods running normally.',
        senderType: 'agent',
        senderName: 'K8sOps',
        senderPersonaType: 'specialist',
        senderIcon: '☸️',
        timestamp: new Date(Date.now() - 3300000), // 55 min ago
      },
      {
        id: 'msg-3',
        content: 'Great! Can you check the status of the production namespace?',
        senderType: 'user',
        senderName: 'You',
        timestamp: new Date(Date.now() - 3000000), // 50 min ago
      },
      {
        id: 'msg-4',
        content: 'Production namespace status:\n• 12 deployments running\n• CPU usage: 45%\n• Memory usage: 62%\n• No errors detected',
        senderType: 'agent',
        senderName: 'K8sOps',
        senderPersonaType: 'specialist',
        senderIcon: '☸️',
        timestamp: new Date(Date.now() - 2700000), // 45 min ago
      },
      {
        id: 'msg-5',
        content: 'Alerting system configured. Monitoring 15 metrics across production.',
        senderType: 'agent',
        senderName: 'SREWatch',
        senderPersonaType: 'analyst',
        senderIcon: '👁️',
        timestamp: new Date(Date.now() - 2400000), // 40 min ago
      },
      {
        id: 'msg-6',
        content: 'Perfect! Keep monitoring and alert me if CPU goes above 80%.',
        senderType: 'user',
        senderName: 'You',
        timestamp: new Date(Date.now() - 2100000), // 35 min ago
      },
      {
        id: 'msg-7',
        content: 'Alert threshold configured: CPU > 80%. Will notify immediately.',
        senderType: 'agent',
        senderName: 'SREWatch',
        senderPersonaType: 'analyst',
        senderIcon: '👁️',
        timestamp: new Date(Date.now() - 1800000), // 30 min ago
      },
      {
        id: 'msg-8',
        content: 'Infrastructure scan complete. 3 security patches available for deployment.',
        senderType: 'agent',
        senderName: 'InfraBot',
        senderPersonaType: 'specialist',
        senderIcon: '🏗️',
        timestamp: new Date(Date.now() - 1200000), // 20 min ago
      },
      {
        id: 'msg-9',
        content: 'Schedule the patches for deployment during the next maintenance window.',
        senderType: 'user',
        senderName: 'You',
        timestamp: new Date(Date.now() - 900000), // 15 min ago
      },
      {
        id: 'msg-10',
        content: 'Patches scheduled for 02:00 UTC tonight. Rollback plan prepared.',
        senderType: 'agent',
        senderName: 'InfraBot',
        senderPersonaType: 'specialist',
        senderIcon: '🏗️',
        timestamp: new Date(Date.now() - 600000), // 10 min ago
      },
    ]

    dispatch(setSquadMembers(mockMembers))
    dispatch(setMessages(mockMessages))
  }, [dispatch])

  /**
   * Handle sending new message
   */
  const handleSend = (content: string) => {
    const currentUser = squadMembers.find(m => m.id === 'user-1')
    if (!currentUser) return

    const newMessage: ChatMessage = {
      id: `msg-${Date.now()}`,
      content,
      senderType: 'user',
      senderName: currentUser.name,
      timestamp: new Date(),
    }

    dispatch(addMessage(newMessage))
  }

  return (
    <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <header className="border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-800 py-4 px-6 flex-shrink-0">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-3">
          Squad Chat
        </h1>
        <div className="max-w-md">
          <Input
            type="text"
            placeholder="Search messages..."
            value={searchQuery}
            onChange={(e) => dispatch(setSearchQuery(e.target.value))}
            icon={<Search className="w-4 h-4" />}
            fullWidth
          />
        </div>
      </header>

      {/* Main content: 2-column layout */}
      <div className="flex flex-1 overflow-hidden">
        {/* Left: Message feed + input */}
        <div className="flex-1 flex flex-col bg-white dark:bg-gray-900">
          <MessageFeed messages={filteredMessages} isLoading={isLoading} />
          <MessageInput onSend={handleSend} disabled={isLoading} />
        </div>

        {/* Right: Squad member sidebar */}
        <SquadMemberList members={squadMembers} searchQuery={searchQuery} />
      </div>
    </div>
  )
}

export default SquadChat
