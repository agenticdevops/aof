import React from 'react'
import { Platform } from '@/types'
import { Slack, MessageCircle, Send, MessageSquare, Github, Layers } from 'lucide-react'
import Card from '@/components/common/Card'
import Badge from '@/components/common/Badge'
import Button from '@/components/common/Button'

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
    <Card elevation="lifted" hoverable>
      <div className="space-y-4">
        <div className="flex items-start justify-between">
          <div>
            <div className="flex items-center gap-3 mb-2">
              <div className="w-10 h-10 rounded-lg bg-sky-100 dark:bg-sky-900 flex items-center justify-center">
                {getIcon()}
              </div>
              <div>
                <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">{platform.name}</h3>
              </div>
            </div>
          </div>
        </div>

        {platform.connected ? (
          <div className="space-y-3">
            <Badge variant="success">Connected</Badge>
            {platform.username && (
              <p className="text-sm text-gray-600 dark:text-gray-400">Connected as @{platform.username}</p>
            )}
          </div>
        ) : (
          <Badge variant="warning">Not connected</Badge>
        )}

        <div className="flex gap-2 pt-4 border-t border-gray-200 dark:border-gray-700">
          <Button
            variant="secondary"
            size="sm"
            fullWidth
            onClick={() => onManage(platform)}
          >
            Manage
          </Button>
          {platform.connected && (
            <Button
              variant="danger"
              size="sm"
              fullWidth
              onClick={() => onDisconnect(platform.id)}
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
