import React from 'react'
import { Card } from '@/components/common/Card'
import { Badge } from '@/components/common/Badge'
import type { SquadMember } from '@/types/chat'

interface SquadMemberListProps {
  members: SquadMember[]
  searchQuery?: string
}

interface MemberCardProps {
  member: SquadMember
}

/**
 * Individual member card in the sidebar
 */
const MemberCard: React.FC<MemberCardProps> = ({ member }) => {
  const { name, role, isOnline, isAgent, personaColor, personaIcon } = member

  return (
    <Card
      elevation="flat"
      className="mb-2 p-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors cursor-pointer"
      clickable
    >
      <div className="flex items-center gap-3">
        {/* Avatar with online indicator */}
        <div className="relative flex-shrink-0">
          <div
            className="w-10 h-10 rounded-full flex items-center justify-center text-lg"
            style={{ backgroundColor: personaColor }}
          >
            {personaIcon}
          </div>
          {/* Online indicator dot */}
          <div
            className={`absolute -top-0.5 -right-0.5 w-3 h-3 rounded-full border-2 border-white dark:border-gray-800 ${
              isOnline ? 'bg-emerald-500' : 'bg-gray-400'
            }`}
            title={isOnline ? 'Online' : 'Offline'}
          />
        </div>

        {/* Member info */}
        <div className="flex-1 min-w-0">
          <div className="flex items-baseline gap-2">
            <span
              className={`font-medium text-sm truncate ${
                isOnline
                  ? 'text-gray-900 dark:text-white'
                  : 'text-gray-500 dark:text-gray-400'
              }`}
            >
              {name}
            </span>
          </div>
          <p className="text-xs text-gray-600 dark:text-gray-400 truncate">
            {role}
          </p>
        </div>

        {/* Agent/Human badge */}
        <div className="flex-shrink-0">
          <Badge
            variant={isAgent ? 'info' : 'neutral'}
            size="sm"
          >
            {isAgent ? '🤖 Agent' : '👤 Human'}
          </Badge>
        </div>
      </div>
    </Card>
  )
}

/**
 * SquadMemberList - Sidebar showing online agents and squad members
 *
 * Features:
 * - Online/offline indicators (green/gray dots)
 * - Agent/Human badges
 * - Persona styling (avatar colors, icons)
 * - Scrollable list for many members
 * - Responsive: hidden on mobile, visible on tablet+
 */
export const SquadMemberList: React.FC<SquadMemberListProps> = ({
  members,
  searchQuery = '',
}) => {
  // Filter members by search query (optional future enhancement)
  const filteredMembers = searchQuery.trim()
    ? members.filter(m =>
        m.name.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : members

  const onlineCount = members.filter(m => m.isOnline).length

  return (
    <div className="hidden lg:flex lg:w-64 border-l border-gray-200 dark:border-gray-800 flex-col bg-white dark:bg-gray-900">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-800">
        <h3 className="font-semibold text-gray-900 dark:text-white">
          Squad Members
        </h3>
        <p className="text-xs text-gray-600 dark:text-gray-400 mt-1">
          {onlineCount} online • {members.length} total
        </p>
      </div>

      {/* Member list */}
      <div className="flex-1 overflow-y-auto p-4">
        {filteredMembers.length === 0 ? (
          <div className="text-center py-8">
            <p className="text-sm text-gray-500 dark:text-gray-400">
              No members found
            </p>
          </div>
        ) : (
          filteredMembers.map((member) => (
            <MemberCard key={member.id} member={member} />
          ))
        )}
      </div>
    </div>
  )
}

export default SquadMemberList
