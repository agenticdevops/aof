import React from 'react'
import { Slack, MessageCircle, Send, Github, Layers, MessageSquare, Settings, Unlink } from 'lucide-react'
import Card from '@/components/common/Card'
import Badge from '@/components/common/Badge'
import Button from '@/components/common/Button'
import { Platform } from '@/types'

interface PlatformCardProps {
  platform: Platform
  onManage: (platform: Platform) => void
  onDisconnect: (id: string) => void
}

export const PlatformCard: React.FC<PlatformCardProps> = ({ platform, onManage, onDisconnect }) => {
  const getIcon = () => {
    const iconProps = { className: 'w-6 h-6 text-sky-600 dark:text-sky-300' }
    switch (platform.type) {
      case 'slack':
        return <Slack {...iconProps} />
      case 'discord':
        return <MessageCircle {...iconProps} />
      case 'telegram':
        return <Send {...iconProps} />
      case 'whatsapp':
        return <MessageSquare {...iconProps} />
      case 'github':
        return <Github {...iconProps} />
      case 'jira':
        return <Layers {...iconProps} />
      default:
        return <MessageCircle {...iconProps} />
    }
  }

  return (
    <Card elevation="lifted">
      <div className="space-y-4">
        {/* Header with icon and status */}
        <div className="flex items-start justify-between">
          <div className="flex items-start gap-3">
            <div className="p-2 bg-sky-100 dark:bg-sky-900/30 rounded-lg">{getIcon()}</div>
            <div>
              <h3 className="font-semibold text-gray-900 dark:text-white capitalize">{platform.name}</h3>
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                {platform.connected ? 'Connected' : 'Not connected'}
              </p>
            </div>
          </div>
          <Badge
            variant={platform.connected ? 'success' : 'warning'}
            size="sm"
            icon={platform.connected ? <Slack className="w-3 h-3" /> : undefined}
          >
            {platform.connected ? 'Active' : 'Inactive'}
          </Badge>
        </div>

        {/* Username if connected */}
        {platform.connected && platform.config?.username && (
          <div className="text-sm">
            <p className="text-gray-500 dark:text-gray-400 mb-1">Username</p>
            <p className="font-medium text-gray-900 dark:text-white">{platform.config.username}</p>
          </div>
        )}

        {/* Actions */}
        <div className="flex gap-2 pt-2 border-t border-gray-200 dark:border-gray-700">
          <Button
            variant="secondary"
            size="sm"
            onClick={() => onManage(platform)}
            icon={<Settings className="w-4 h-4" />}
            iconPosition="left"
            fullWidth
          >
            Manage
          </Button>
          {platform.connected && (
            <Button
              variant="danger"
              size="sm"
              onClick={() => onDisconnect(platform.id)}
              icon={<Unlink className="w-4 h-4" />}
              iconPosition="left"
              fullWidth
            >
              Disconnect
            </Button>
          )}
        </div>
      </div>
    </Card>
  )
}

export default PlatformCard
