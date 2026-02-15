import React, { useState } from 'react'
import { useAppDispatch, useAppSelector, addWizardPlatform, removeWizardPlatform } from '@/store'
import { Slack, MessageCircle, Send, Github, Layers, MessageSquare } from 'lucide-react'
import Button from '@/components/common/Button'
import Card from '@/components/common/Card'
import Modal from '@/components/common/Modal'
import Input from '@/components/common/Input'
import Badge from '@/components/common/Badge'

interface StepPlatformConfigProps {
  onNext: () => void
  onBack: () => void
}

interface Platform {
  id: string
  name: string
  icon: React.ReactNode
  description: string
}

const platforms: Platform[] = [
  {
    id: 'slack',
    name: 'Slack',
    icon: <Slack className="w-6 h-6" />,
    description: 'Connect to Slack workspace',
  },
  {
    id: 'discord',
    name: 'Discord',
    icon: <MessageCircle className="w-6 h-6" />,
    description: 'Add to Discord server',
  },
  {
    id: 'telegram',
    name: 'Telegram',
    icon: <Send className="w-6 h-6" />,
    description: 'Bot for Telegram chat',
  },
  {
    id: 'whatsapp',
    name: 'WhatsApp',
    icon: <MessageSquare className="w-6 h-6" />,
    description: 'WhatsApp Business API',
  },
  {
    id: 'github',
    name: 'GitHub',
    icon: <Github className="w-6 h-6" />,
    description: 'Access to repositories',
  },
  {
    id: 'jira',
    name: 'Jira',
    icon: <Layers className="w-6 h-6" />,
    description: 'Project management sync',
  },
]

export const StepPlatformConfig: React.FC<StepPlatformConfigProps> = ({ onNext, onBack }) => {
  const dispatch = useAppDispatch()
  const { platforms: connectedPlatforms } = useAppSelector((state) => state.onboarding)
  const [selectedPlatform, setSelectedPlatform] = useState<string | null>(null)
  const [showModal, setShowModal] = useState(false)
  const [token, setToken] = useState('')
  const [testingPlatform, setTestingPlatform] = useState<string | null>(null)
  const [testResult, setTestResult] = useState<{ platform: string; success: boolean; message: string } | null>(null)

  const handleConnectClick = (platformId: string) => {
    setSelectedPlatform(platformId)
    setToken('')
    setTestResult(null)
    setShowModal(true)
  }

  const handleTestConnection = async () => {
    if (!selectedPlatform || !token) return

    setTestingPlatform(selectedPlatform)
    // Simulate API call
    await new Promise((resolve) => setTimeout(resolve, 1500))

    setTestResult({
      platform: selectedPlatform,
      success: true,
      message: `Successfully connected to ${platforms.find((p) => p.id === selectedPlatform)?.name}`,
    })
    setTestingPlatform(null)
  }

  const handleConfirmConnection = () => {
    if (!selectedPlatform || !token) return

    dispatch(
      addWizardPlatform({
        key: selectedPlatform,
        platform: {
          connected: true,
          username: token.substring(0, 10) + '...',
          config: { token },
        },
      })
    )

    setShowModal(false)
    setSelectedPlatform(null)
    setToken('')
    setTestResult(null)
  }

  const handleDisconnect = (platformId: string) => {
    dispatch(removeWizardPlatform(platformId))
  }

  const isConnected = (platformId: string) => connectedPlatforms[platformId]?.connected

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">Connect your platforms</h2>
        <p className="text-gray-600 dark:text-gray-400">Choose where your agent should listen for messages</p>
      </div>

      <div className="grid grid-cols-2 gap-4">
        {platforms.map((platform) => (
          <Card key={platform.id} elevation={isConnected(platform.id) ? 'focused' : 'lifted'} hoverable>
            <div className="space-y-4">
              <div className="flex items-start justify-between">
                <div className="flex items-start gap-3">
                  <div className="p-2 bg-sky-100 dark:bg-sky-900/30 rounded-lg text-sky-600 dark:text-sky-300">
                    {platform.icon}
                  </div>
                  <div>
                    <h3 className="font-semibold text-gray-900 dark:text-white">{platform.name}</h3>
                    <p className="text-xs text-gray-500 dark:text-gray-400">{platform.description}</p>
                  </div>
                </div>
                {isConnected(platform.id) && (
                  <Badge variant="success" size="sm">
                    Connected
                  </Badge>
                )}
              </div>

              {isConnected(platform.id) ? (
                <Button variant="secondary" size="sm" onClick={() => handleDisconnect(platform.id)} fullWidth>
                  Disconnect
                </Button>
              ) : (
                <Button variant="primary" size="sm" onClick={() => handleConnectClick(platform.id)} fullWidth>
                  Connect
                </Button>
              )}
            </div>
          </Card>
        ))}
      </div>

      {/* Connection Modal */}
      <Modal
        isOpen={showModal}
        onClose={() => setShowModal(false)}
        title={`Connect ${selectedPlatform ? platforms.find((p) => p.id === selectedPlatform)?.name : ''}`}
        size="md"
        footer={
          <div className="flex gap-3 justify-end">
            <Button variant="secondary" onClick={() => setShowModal(false)} disabled={testingPlatform !== null}>
              Cancel
            </Button>
            {!testResult?.success ? (
              <Button
                variant="primary"
                onClick={handleTestConnection}
                loading={testingPlatform !== null}
                disabled={!token || testingPlatform !== null}
              >
                Test Connection
              </Button>
            ) : (
              <Button variant="primary" onClick={handleConfirmConnection}>
                Confirm Connection
              </Button>
            )}
          </div>
        }
      >
        <div className="space-y-4">
          <Input
            label="Authentication Token"
            placeholder="Paste your API token or authentication key"
            type="password"
            value={token}
            onChange={(e) => setToken(e.target.value)}
            helperText="Your token is never stored or shared"
            fullWidth
          />

          {testResult && (
            <div
              className={`p-4 rounded-lg ${
                testResult.success
                  ? 'bg-green-50 dark:bg-green-900/20 text-green-800 dark:text-green-300'
                  : 'bg-red-50 dark:bg-red-900/20 text-red-800 dark:text-red-300'
              }`}
            >
              {testResult.message}
            </div>
          )}
        </div>
      </Modal>

      <div className="flex gap-3 justify-end">
        <Button variant="secondary" onClick={onBack}>
          Back
        </Button>
        <Button variant="primary" onClick={onNext}>
          Next
        </Button>
      </div>
    </div>
  )
}

export default StepPlatformConfig
