import React, { useState } from 'react'
import { useAppDispatch, useAppSelector, setNavigation } from '@/store'
import { ChevronDown, FolderOpen, Bot, Globe, CheckCircle } from 'lucide-react'
import Button from '@/components/common/Button'
import Card from '@/components/common/Card'
import Badge from '@/components/common/Badge'

interface StepReviewProps {
  onBack: () => void
}

type SectionKey = 'project' | 'agent' | 'platforms'

export const StepReview: React.FC<StepReviewProps> = ({ onBack }) => {
  const dispatch = useAppDispatch()
  const { project, agent, platforms } = useAppSelector((state) => state.onboarding)
  const [expandedSections, setExpandedSections] = useState<Record<SectionKey, boolean>>({
    project: true,
    agent: true,
    platforms: true,
  })

  const toggleSection = (section: SectionKey) => {
    setExpandedSections((prev) => ({
      ...prev,
      [section]: !prev[section],
    }))
  }

  const handleLaunch = () => {
    dispatch(setNavigation('config'))
  }

  const connectedPlatforms = Object.entries(platforms).filter(([, p]) => p.connected).length

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">Review and launch</h2>
        <p className="text-gray-600 dark:text-gray-400">
          Review your configuration before launching your agent squad
        </p>
      </div>

      {/* Project Section */}
      <Card elevation="lifted">
        <button
          onClick={() => toggleSection('project')}
          className="w-full flex items-center justify-between hover:opacity-80 transition-opacity"
        >
          <div className="flex items-center gap-3">
            <FolderOpen className="w-5 h-5 text-sky-500" />
            <h3 className="font-semibold text-gray-900 dark:text-white">Project</h3>
          </div>
          <ChevronDown
            className={`w-5 h-5 text-gray-400 transition-transform duration-200 ${
              expandedSections.project ? 'rotate-180' : ''
            }`}
          />
        </button>
        {expandedSections.project && (
          <div className="mt-4 space-y-3 pt-4 border-t border-gray-200 dark:border-gray-700">
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Name</p>
              <p className="font-medium text-gray-900 dark:text-white">{project.name}</p>
            </div>
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Description</p>
              <p className="text-gray-700 dark:text-gray-300">{project.description}</p>
            </div>
          </div>
        )}
      </Card>

      {/* Agent Section */}
      <Card elevation="lifted">
        <button
          onClick={() => toggleSection('agent')}
          className="w-full flex items-center justify-between hover:opacity-80 transition-opacity"
        >
          <div className="flex items-center gap-3">
            <Bot className="w-5 h-5 text-sky-500" />
            <h3 className="font-semibold text-gray-900 dark:text-white">Agent Configuration</h3>
          </div>
          <ChevronDown
            className={`w-5 h-5 text-gray-400 transition-transform duration-200 ${
              expandedSections.agent ? 'rotate-180' : ''
            }`}
          />
        </button>
        {expandedSections.agent && (
          <div className="mt-4 space-y-3 pt-4 border-t border-gray-200 dark:border-gray-700">
            <div className="flex items-center justify-between">
              <p className="text-sm text-gray-500 dark:text-gray-400">Name</p>
              <Badge variant="info" size="sm">
                {agent.name}
              </Badge>
            </div>
            <div className="flex items-center justify-between">
              <p className="text-sm text-gray-500 dark:text-gray-400">Model</p>
              <Badge variant="neutral" size="sm">
                {agent.model}
              </Badge>
            </div>
            <div className="flex items-center justify-between">
              <p className="text-sm text-gray-500 dark:text-gray-400">Type</p>
              <Badge variant="info" size="sm" className="capitalize">
                {agent.type}
              </Badge>
            </div>
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400 mb-2">Capabilities</p>
              <div className="flex flex-wrap gap-2">
                {agent.capabilities.map((cap) => (
                  <Badge key={cap} variant="success" size="sm">
                    {cap}
                  </Badge>
                ))}
              </div>
            </div>
          </div>
        )}
      </Card>

      {/* Platforms Section */}
      <Card elevation="lifted">
        <button
          onClick={() => toggleSection('platforms')}
          className="w-full flex items-center justify-between hover:opacity-80 transition-opacity"
        >
          <div className="flex items-center gap-3">
            <Globe className="w-5 h-5 text-sky-500" />
            <h3 className="font-semibold text-gray-900 dark:text-white">Platform Connections</h3>
            <Badge variant="info" size="sm">
              {connectedPlatforms}
            </Badge>
          </div>
          <ChevronDown
            className={`w-5 h-5 text-gray-400 transition-transform duration-200 ${
              expandedSections.platforms ? 'rotate-180' : ''
            }`}
          />
        </button>
        {expandedSections.platforms && (
          <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
            {connectedPlatforms > 0 ? (
              <div className="space-y-2">
                {Object.entries(platforms).map(
                  ([key, platform]) =>
                    platform.connected && (
                      <div key={key} className="flex items-center justify-between p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
                        <div className="flex items-center gap-2">
                          <CheckCircle className="w-4 h-4 text-green-600 dark:text-green-400" />
                          <span className="font-medium text-gray-900 dark:text-white capitalize">{key}</span>
                        </div>
                        {platform.username && (
                          <span className="text-sm text-gray-500 dark:text-gray-400">{platform.username}</span>
                        )}
                      </div>
                    )
                )}
              </div>
            ) : (
              <p className="text-sm text-gray-500 dark:text-gray-400">No platforms connected yet</p>
            )}
          </div>
        )}
      </Card>

      <div className="bg-blue-50 dark:bg-blue-900/20 p-4 rounded-lg">
        <p className="text-sm text-blue-900 dark:text-blue-300">
          Once you launch, your agent will be live and ready to handle messages from connected platforms.
        </p>
      </div>

      <div className="flex gap-3 justify-end">
        <Button variant="secondary" onClick={onBack}>
          Back
        </Button>
        <Button variant="primary" onClick={handleLaunch}>
          Launch Agent
        </Button>
      </div>
    </div>
  )
}

export default StepReview
