import React, { useState } from 'react'
import { useAppDispatch, useAppSelector, setSearchQuery } from '@/store'
import { Search, Plus, Edit2, Trash2, Settings, Link as LinkIcon, CheckCircle, AlertCircle } from 'lucide-react'
import TabNavigation from '@/components/config/TabNavigation'
import AgentCard from '@/components/config/AgentCard'
import PlatformCard from '@/components/config/PlatformCard'
import SearchBar from '@/components/common/SearchBar'
import Button from '@/components/common/Button'
import EmptyState from '@/components/common/EmptyState'
import Card from '@/components/common/Card'
import Badge from '@/components/common/Badge'
import Modal from '@/components/common/Modal'
import Input from '@/components/common/Input'
import TextArea from '@/components/common/TextArea'
import Select from '@/components/common/Select'
import Checkbox from '@/components/common/Checkbox'
import ConfirmDialog from '@/components/common/ConfirmDialog'
import { Agent, Platform, Tool } from '@/types'

export const ConfigurationPage: React.FC = () => {
  const dispatch = useAppDispatch()
  const { agents, tools, platforms, searchQuery } = useAppSelector((state) => state.config)
  const [activeTab, setActiveTab] = useState<'agents' | 'tools' | 'platforms'>('agents')
  const [editingAgent, setEditingAgent] = useState<Agent | null>(null)
  const [showAgentModal, setShowAgentModal] = useState(false)
  const [showConfirmDelete, setShowConfirmDelete] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  // Mock data for tools
  const mockTools: Tool[] = [
    {
      id: '1',
      name: 'Shell Commands',
      description: 'Execute shell commands on the system',
      type: 'api',
      status: 'ready',
    },
    {
      id: '2',
      name: 'HTTP Requests',
      description: 'Make HTTP requests to external APIs',
      type: 'integration',
      status: 'ready',
    },
    {
      id: '3',
      name: 'File Operations',
      description: 'Read and write files on the system',
      type: 'script',
      status: 'needs-config',
    },
  ]

  const filteredAgents = agents.filter(
    (agent) =>
      agent.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      agent.model.toLowerCase().includes(searchQuery.toLowerCase())
  )

  const handleEditAgent = (agent: Agent) => {
    setEditingAgent(agent)
    setShowAgentModal(true)
  }

  const handleDeleteAgent = (id: string) => {
    setDeleteTarget(id)
    setShowConfirmDelete(true)
  }

  const confirmDelete = async () => {
    if (deleteTarget) {
      setIsLoading(true)
      // API call would go here
      setTimeout(() => {
        setIsLoading(false)
        setShowConfirmDelete(false)
        setDeleteTarget(null)
      }, 1000)
    }
  }

  const handleCreateAgent = () => {
    setEditingAgent(null)
    setShowAgentModal(true)
  }

  const handleSaveAgent = async () => {
    setIsLoading(true)
    // API call would go here
    setTimeout(() => {
      setIsLoading(false)
      setShowAgentModal(false)
      setEditingAgent(null)
    }, 1000)
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 sticky top-0 z-10">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">Configuration</h1>
              <p className="text-gray-600 dark:text-gray-400 mt-1">Manage your agents, tools, and platform connections</p>
            </div>
            {activeTab === 'agents' && (
              <Button onClick={handleCreateAgent} icon={<Plus className="w-5 h-5" />} iconPosition="left">
                Create Agent
              </Button>
            )}
          </div>

          <TabNavigation activeTab={activeTab} onTabChange={setActiveTab} />
        </div>
      </div>

      {/* Content */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Agents Tab */}
        {activeTab === 'agents' && (
          <div className="space-y-6">
            {agents.length > 0 && (
              <SearchBar
                value={searchQuery}
                onChange={(value) => dispatch(setSearchQuery(value))}
                placeholder="Search agents..."
                fullWidth
              />
            )}

            {filteredAgents.length > 0 ? (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {filteredAgents.map((agent) => (
                  <AgentCard
                    key={agent.id}
                    agent={agent}
                    onEdit={handleEditAgent}
                    onDelete={handleDeleteAgent}
                  />
                ))}
              </div>
            ) : agents.length > 0 ? (
              <EmptyState
                icon={<Search className="w-16 h-16" />}
                title="No agents found"
                description={`No agents match "${searchQuery}"`}
              />
            ) : (
              <EmptyState
                icon={<Plus className="w-16 h-16" />}
                title="No agents yet"
                description="Create your first agent to get started"
                action={<Button onClick={handleCreateAgent}>Create Agent</Button>}
              />
            )}
          </div>
        )}

        {/* Tools Tab */}
        {activeTab === 'tools' && (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {mockTools.map((tool) => (
                <Card key={tool.id} elevation="lifted">
                  <div className="space-y-4">
                    <div>
                      <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">{tool.name}</h3>
                      <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">{tool.description}</p>
                    </div>

                    <div className="flex gap-2">
                      <Badge
                        variant={tool.type === 'api' ? 'info' : tool.type === 'integration' ? 'warning' : 'neutral'}
                        size="sm"
                      >
                        {tool.type}
                      </Badge>
                      <Badge
                        variant={tool.status === 'ready' ? 'success' : 'warning'}
                        size="sm"
                        icon={tool.status === 'ready' ? <CheckCircle className="w-4 h-4" /> : <AlertCircle className="w-4 h-4" />}
                      >
                        {tool.status === 'ready' ? 'Ready' : 'Needs Config'}
                      </Badge>
                    </div>

                    <Button variant="secondary" size="sm" fullWidth>
                      {tool.status === 'ready' ? 'View Details' : 'Configure'}
                    </Button>
                  </div>
                </Card>
              ))}
            </div>
          </div>
        )}

        {/* Platforms Tab */}
        {activeTab === 'platforms' && (
          <div className="space-y-6">
            {platforms.length > 0 ? (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {platforms.map((platform) => (
                  <PlatformCard
                    key={platform.id}
                    platform={platform}
                    onManage={() => {}}
                    onDisconnect={() => {}}
                  />
                ))}
              </div>
            ) : (
              <EmptyState
                icon={<LinkIcon className="w-16 h-16" />}
                title="No platforms connected"
                description="Connect your first platform to start receiving messages"
              />
            )}
          </div>
        )}
      </div>

      {/* Agent Edit Modal */}
      <Modal
        isOpen={showAgentModal}
        onClose={() => {
          setShowAgentModal(false)
          setEditingAgent(null)
        }}
        title={editingAgent ? `Edit ${editingAgent.name}` : 'Create New Agent'}
        size="lg"
        footer={
          <div className="flex gap-3 justify-end">
            <Button
              variant="secondary"
              onClick={() => {
                setShowAgentModal(false)
                setEditingAgent(null)
              }}
              disabled={isLoading}
            >
              Cancel
            </Button>
            <Button variant="primary" onClick={handleSaveAgent} loading={isLoading} disabled={isLoading}>
              {editingAgent ? 'Update' : 'Create'}
            </Button>
          </div>
        }
      >
        <div className="space-y-4">
          <Input label="Agent Name" placeholder="My Agent" fullWidth />
          <Select
            label="Model"
            options={[
              { value: 'claude', label: 'Claude 3.5 Sonnet' },
              { value: 'gpt-4', label: 'GPT-4' },
              { value: 'gemini', label: 'Gemini 2.5' },
            ]}
            fullWidth
          />
          <TextArea label="Instructions" placeholder="Agent instructions..." rows={4} fullWidth />
          <div className="space-y-2">
            <p className="text-sm font-medium text-gray-700 dark:text-gray-300">Capabilities</p>
            <Checkbox label="Shell Commands" />
            <Checkbox label="HTTP Requests" />
            <Checkbox label="File Operations" />
          </div>
        </div>
      </Modal>

      {/* Confirm Delete Dialog */}
      <ConfirmDialog
        isOpen={showConfirmDelete}
        title="Delete Agent"
        message="Are you sure you want to delete this agent? This action cannot be undone."
        confirmText="Delete"
        cancelText="Cancel"
        isDangerous
        isLoading={isLoading}
        onConfirm={confirmDelete}
        onCancel={() => {
          setShowConfirmDelete(false)
          setDeleteTarget(null)
        }}
      />
    </div>
  )
}

export default ConfigurationPage
