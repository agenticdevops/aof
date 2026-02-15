import React, { useState } from 'react'
import { useAppDispatch, useAppSelector, updateChannels } from '@/store'
import { Channel } from '@/types/onboarding'
import Input from '@/components/common/Input'
import Checkbox from '@/components/common/Checkbox'
import Badge from '@/components/common/Badge'
import LoadingSpinner from '@/components/common/LoadingSpinner'

/**
 * Channel configuration form state
 */
interface ChannelConfigs {
  slack: {
    webhookUrl: string
    botToken: string
    channelName: string
  }
  telegram: {
    botToken: string
    chatId: string
  }
  discord: {
    serverId: string
    botToken: string
    channelId: string
  }
}

/**
 * Step 1: Channel Selection and Configuration
 * Allows users to select which platforms Xops will connect to
 */
export const StepChannels: React.FC = () => {
  const dispatch = useAppDispatch()
  const selectedChannels = useAppSelector((state) => state.onboarding.selectedChannels)
  const loading = useAppSelector((state) => state.onboarding.loading)

  const [channelConfigs, setChannelConfigs] = useState<ChannelConfigs>({
    slack: { webhookUrl: '', botToken: '', channelName: '' },
    telegram: { botToken: '', chatId: '' },
    discord: { serverId: '', botToken: '', channelId: '' },
  })

  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({})
  const [testingChannel, setTestingChannel] = useState<Channel | null>(null)
  const [testResults, setTestResults] = useState<Record<Channel, boolean>>({})

  const handleChannelToggle = (channel: Channel) => {
    const updated = selectedChannels.includes(channel)
      ? selectedChannels.filter((c) => c !== channel)
      : [...selectedChannels, channel]

    dispatch(updateChannels(updated))
    // Clear errors when toggling
    setValidationErrors({})
  }

  const handleConfigChange = (channel: 'slack' | 'telegram' | 'discord', field: string, value: string) => {
    setChannelConfigs((prev) => ({
      ...prev,
      [channel]: { ...prev[channel], [field]: value },
    }))
    // Clear validation error for this field
    setValidationErrors((prev) => {
      const newErrors = { ...prev }
      delete newErrors[`${channel}.${field}`]
      return newErrors
    })
  }

  // Validation functions
  const validateSlackToken = (token: string): boolean => {
    return token.startsWith('xoxb-') || token.startsWith('xoxp-')
  }

  const validateTelegramToken = (token: string): boolean => {
    return token.includes(':') && token.split(':').length === 2
  }

  const validateDiscordToken = (token: string): boolean => {
    return token.length > 0 && !token.includes(' ')
  }

  const validateDiscordId = (id: string): boolean => {
    return /^\d+$/.test(id)
  }

  const validateTelegramChatId = (id: string): boolean => {
    return /^-?\d+$/.test(id) || id.startsWith('@')
  }

  const handleTestConnection = async (channel: Channel) => {
    setTestingChannel(channel)

    try {
      // Validate locally first
      switch (channel) {
        case 'slack': {
          const { botToken } = channelConfigs.slack
          if (!botToken) {
            setValidationErrors((prev) => ({
              ...prev,
              'slack.botToken': 'Bot token is required',
            }))
            setTestingChannel(null)
            return
          }
          if (!validateSlackToken(botToken)) {
            setValidationErrors((prev) => ({
              ...prev,
              'slack.botToken': "Slack token must start with 'xoxb-' or 'xoxp-'",
            }))
            setTestingChannel(null)
            return
          }
          break
        }
        case 'telegram': {
          const { botToken, chatId } = channelConfigs.telegram
          if (!botToken) {
            setValidationErrors((prev) => ({
              ...prev,
              'telegram.botToken': 'Bot token is required',
            }))
            setTestingChannel(null)
            return
          }
          if (!validateTelegramToken(botToken)) {
            setValidationErrors((prev) => ({
              ...prev,
              'telegram.botToken': "Telegram token format invalid (should be like '123456:ABC-DEF')",
            }))
            setTestingChannel(null)
            return
          }
          if (!chatId) {
            setValidationErrors((prev) => ({
              ...prev,
              'telegram.chatId': 'Chat ID is required',
            }))
            setTestingChannel(null)
            return
          }
          if (!validateTelegramChatId(chatId)) {
            setValidationErrors((prev) => ({
              ...prev,
              'telegram.chatId': 'Chat ID must be numeric or start with @',
            }))
            setTestingChannel(null)
            return
          }
          break
        }
        case 'discord': {
          const { serverId, botToken, channelId } = channelConfigs.discord
          if (!serverId || !botToken || !channelId) {
            if (!serverId) {
              setValidationErrors((prev) => ({
                ...prev,
                'discord.serverId': 'Server ID is required',
              }))
            }
            if (!botToken) {
              setValidationErrors((prev) => ({
                ...prev,
                'discord.botToken': 'Bot token is required',
              }))
            }
            if (!channelId) {
              setValidationErrors((prev) => ({
                ...prev,
                'discord.channelId': 'Channel ID is required',
              }))
            }
            setTestingChannel(null)
            return
          }
          if (!validateDiscordId(serverId)) {
            setValidationErrors((prev) => ({
              ...prev,
              'discord.serverId': 'Server ID must be numeric',
            }))
            setTestingChannel(null)
            return
          }
          if (!validateDiscordId(channelId)) {
            setValidationErrors((prev) => ({
              ...prev,
              'discord.channelId': 'Channel ID must be numeric',
            }))
            setTestingChannel(null)
            return
          }
          if (!validateDiscordToken(botToken)) {
            setValidationErrors((prev) => ({
              ...prev,
              'discord.botToken': 'Bot token format invalid',
            }))
            setTestingChannel(null)
            return
          }
          break
        }
      }

      // Simulate connection test
      await new Promise((resolve) => setTimeout(resolve, 1000))
      setTestResults((prev) => ({
        ...prev,
        [channel]: true,
      }))
      setValidationErrors({})
    } catch (error: any) {
      setValidationErrors((prev) => ({
        ...prev,
        [channel]: error.message || 'Connection test failed',
      }))
    } finally {
      setTestingChannel(null)
    }
  }

  const renderChannelConfig = (channel: 'slack' | 'telegram' | 'discord') => {
    if (!selectedChannels.includes(channel)) return null

    const config = channelConfigs[channel]
    const errorPrefix = `${channel}.`

    return (
      <div key={channel} className="mt-4 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
        {channel === 'slack' && (
          <>
            <Input
              label="Slack Bot Token"
              placeholder="xoxb-... or xoxp-..."
              value={config.botToken}
              onChange={(e) => handleConfigChange('slack', 'botToken', e.target.value)}
              error={validationErrors[`${errorPrefix}botToken`]}
              helperText="Get your bot token from Slack API dashboard"
              fullWidth
            />
            <div className="mt-3">
              <Input
                label="Channel Name"
                placeholder="e.g., #ops or #xops-alerts"
                value={config.channelName}
                onChange={(e) => handleConfigChange('slack', 'channelName', e.target.value)}
                error={validationErrors[`${errorPrefix}channelName`]}
                fullWidth
              />
            </div>
          </>
        )}

        {channel === 'telegram' && (
          <>
            <Input
              label="Telegram Bot Token"
              placeholder="123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
              value={config.botToken}
              onChange={(e) => handleConfigChange('telegram', 'botToken', e.target.value)}
              error={validationErrors[`${errorPrefix}botToken`]}
              helperText="Get from BotFather on Telegram"
              fullWidth
            />
            <div className="mt-3">
              <Input
                label="Chat ID or Channel"
                placeholder="-123456789 or @my_channel"
                value={config.chatId}
                onChange={(e) => handleConfigChange('telegram', 'chatId', e.target.value)}
                error={validationErrors[`${errorPrefix}chatId`]}
                fullWidth
              />
            </div>
          </>
        )}

        {channel === 'discord' && (
          <>
            <Input
              label="Server ID (Guild ID)"
              placeholder="123456789012345678"
              value={config.serverId}
              onChange={(e) => handleConfigChange('discord', 'serverId', e.target.value)}
              error={validationErrors[`${errorPrefix}serverId`]}
              fullWidth
            />
            <div className="mt-3">
              <Input
                label="Bot Token"
                placeholder="MzX...NzE"
                value={config.botToken}
                onChange={(e) => handleConfigChange('discord', 'botToken', e.target.value)}
                error={validationErrors[`${errorPrefix}botToken`]}
                fullWidth
              />
            </div>
            <div className="mt-3">
              <Input
                label="Channel ID"
                placeholder="123456789012345678"
                value={config.channelId}
                onChange={(e) => handleConfigChange('discord', 'channelId', e.target.value)}
                error={validationErrors[`${errorPrefix}channelId`]}
                fullWidth
              />
            </div>
          </>
        )}

        <div className="mt-3 flex gap-2">
          <button
            onClick={() => handleTestConnection(channel)}
            disabled={testingChannel === channel}
            className="px-3 py-2 text-sm font-medium text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded disabled:opacity-50"
          >
            {testingChannel === channel ? 'Testing...' : 'Test Connection'}
          </button>
          {testResults[channel] && (
            <Badge variant="success">✓ Connected</Badge>
          )}
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
          Where should Xops appear?
        </h2>
        <p className="text-gray-600 dark:text-gray-400">
          Select the platforms where you want Xops to coordinate with your team
        </p>
      </div>

      {loading && <LoadingSpinner message="Checking connectivity..." />}

      <div className="space-y-4">
        {/* Slack */}
        <div className="border border-gray-200 dark:border-gray-700 rounded-lg p-4 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition">
          <label className="flex items-start cursor-pointer">
            <Checkbox
              checked={selectedChannels.includes('slack')}
              onChange={() => handleChannelToggle('slack')}
            />
            <div className="ml-3 flex-1">
              <div className="font-semibold text-gray-900 dark:text-white">Slack</div>
              <div className="text-sm text-gray-600 dark:text-gray-400">
                Connect to your Slack workspace
              </div>
            </div>
          </label>
          {renderChannelConfig('slack')}
        </div>

        {/* Telegram */}
        <div className="border border-gray-200 dark:border-gray-700 rounded-lg p-4 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition">
          <label className="flex items-start cursor-pointer">
            <Checkbox
              checked={selectedChannels.includes('telegram')}
              onChange={() => handleChannelToggle('telegram')}
            />
            <div className="ml-3 flex-1">
              <div className="font-semibold text-gray-900 dark:text-white">Telegram</div>
              <div className="text-sm text-gray-600 dark:text-gray-400">
                Connect to a Telegram bot or channel
              </div>
            </div>
          </label>
          {renderChannelConfig('telegram')}
        </div>

        {/* Discord */}
        <div className="border border-gray-200 dark:border-gray-700 rounded-lg p-4 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition">
          <label className="flex items-start cursor-pointer">
            <Checkbox
              checked={selectedChannels.includes('discord')}
              onChange={() => handleChannelToggle('discord')}
            />
            <div className="ml-3 flex-1">
              <div className="font-semibold text-gray-900 dark:text-white">Discord</div>
              <div className="text-sm text-gray-600 dark:text-gray-400">
                Connect to a Discord server
              </div>
            </div>
          </label>
          {renderChannelConfig('discord')}
        </div>
      </div>

      {selectedChannels.length === 0 && (
        <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-700 dark:text-red-200 text-sm">
          At least one communication channel is required for Xops to operate
        </div>
      )}
    </div>
  )
}

export default StepChannels
