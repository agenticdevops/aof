import React, { useState } from 'react'
import { useAppDispatch, useAppSelector, addPlatform, removePlatform } from '@/store'
import Card from '@/components/common/Card'
import Input from '@/components/common/Input'
import Button from '@/components/common/Button'
import Modal from '@/components/common/Modal'
import Badge from '@/components/common/Badge'

interface StepPlatformConfigProps {
  onBack: () => void
  onNext: () => void
}

const platforms = [
  { type: 'slack', name: 'Slack', icon: '💬' },
  { type: 'discord', name: 'Discord', icon: '🎮' },
  { type: 'telegram', name: 'Telegram', icon: '✈️' },
  { type: 'whatsapp', name: 'WhatsApp', icon: '📱' },
  { type: 'github', name: 'GitHub', icon: '🐙' },
  { type: 'jira', name: 'Jira', icon: '📋' },
]

interface PlatformModalState {
  isOpen: boolean
  platform: (typeof platforms)[0] | null
  configValue: string
  isTesting: boolean
  testResult: 'success' | 'error' | null
}

export const StepPlatformConfig: React.FC<StepPlatformConfigProps> = ({ onBack, onNext }) => {
  const dispatch = useAppDispatch()
  const { platforms: connectedPlatforms } = useAppSelector((state) => state.onboarding)
  const [modalState, setModalState] = useState<PlatformModalState>({
    isOpen: false,
    platform: null,
    configValue: '',
    isTesting: false,
    testResult: null,
  })

  const openModal = (platform: (typeof platforms)[0]) => {
    setModalState({
      isOpen: true,
      platform,
      configValue: '',
      isTesting: false,
      testResult: null,
    })
  }

  const closeModal = () => {
    setModalState({ isOpen: false, platform: null, configValue: '', isTesting: false, testResult: null })
  }

  const handleTestConnection = async () => {
    setModalState((prev) => ({ ...prev, isTesting: true }))
    // Simulate API call
    setTimeout(() => {
      setModalState((prev) => ({ ...prev, isTesting: false, testResult: 'success' }))
    }, 1500)
  }

  const handleSave = () => {
    if (modalState.platform && modalState.configValue) {
      dispatch(
        addPlatform({
          type: modalState.platform.type,
          config: {
            type: modalState.platform.type as any,
            name: modalState.platform.name,
            connected: true,
            username: modalState.configValue.split(':')[0],
            config: { token: modalState.configValue },
          },
        })
      )
      closeModal()
    }
  }

  const handleRemove = (platformType: string) => {
    dispatch(removePlatform(platformType))
  }

  const isConnected = (platformType: string) => {
    return connectedPlatforms[platformType]?.connected || false
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">Where should your agent listen?</h2>
        <p className="text-gray-600 dark:text-gray-400">Connect platforms where your agent will receive messages (optional).</p>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        {platforms.map((platform) => (
          <Card
            key={platform.type}
            className="relative"
            elevation={isConnected(platform.type) ? 'focused' : 'lifted'}
            hoverable
          >
            <div className="space-y-4">
              <div className="flex items-start justify-between">
                <div className="text-4xl">{platform.icon}</div>
                {isConnected(platform.type) && <Badge variant="success" size="sm">Connected</Badge>}
              </div>

              <div>
                <h3 className="font-semibold text-gray-900 dark:text-gray-100">{platform.name}</h3>
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">Connect your {platform.name} account</p>
              </div>

              {isConnected(platform.type) ? (
                <div className="space-y-2">
                  <p className="text-xs text-gray-600 dark:text-gray-400">
                    ✓ Connected as @{connectedPlatforms[platform.type]?.username}
                  </p>
                  <Button
                    variant="secondary"
                    size="sm"
                    fullWidth
                    onClick={() => handleRemove(platform.type)}
                  >
                    Disconnect
                  </Button>
                </div>
              ) : (
                <Button
                  variant="primary"
                  size="sm"
                  fullWidth
                  onClick={() => openModal(platform)}
                >
                  Connect
                </Button>
              )}
            </div>
          </Card>
        ))}
      </div>

      {/* Connection Modal */}
      <Modal
        isOpen={modalState.isOpen}
        onClose={closeModal}
        title={`Connect ${modalState.platform?.name}`}
        size="md"
        footer={
          <div className="flex gap-3 justify-end">
            <Button variant="secondary" onClick={closeModal} disabled={modalState.isTesting}>
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={handleTestConnection}
              loading={modalState.isTesting}
              disabled={!modalState.configValue || modalState.isTesting}
            >
              Test Connection
            </Button>
          </div>
        }
      >
        <div className="space-y-4">
          <Input
            label={`${modalState.platform?.name} Token/Key`}
            placeholder="Enter your API token or authentication key"
            value={modalState.configValue}
            onChange={(e) => setModalState((prev) => ({ ...prev, configValue: e.target.value }))}
            type="password"
            fullWidth
          />

          {modalState.testResult === 'success' && (
            <div className="p-3 bg-emerald-50 dark:bg-emerald-900 border border-emerald-200 dark:border-emerald-700 rounded-lg">
              <p className="text-sm text-emerald-800 dark:text-emerald-200">✓ Connection successful! Click Save to continue.</p>
              <Button variant="primary" size="sm" fullWidth className="mt-3" onClick={handleSave}>
                Save Connection
              </Button>
            </div>
          )}

          {modalState.testResult === 'error' && (
            <div className="p-3 bg-red-50 dark:bg-red-900 border border-red-200 dark:border-red-700 rounded-lg">
              <p className="text-sm text-red-800 dark:text-red-200">✗ Connection failed. Check your token and try again.</p>
            </div>
          )}
        </div>
      </Modal>

      <div className="flex justify-between gap-3 pt-4">
        <Button variant="secondary" onClick={onBack}>
          Back
        </Button>
        <Button onClick={onNext}>Next</Button>
      </div>
    </div>
  )
}

export default StepPlatformConfig
