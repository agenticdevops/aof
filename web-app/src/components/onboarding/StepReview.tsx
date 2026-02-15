import React, { useState } from 'react'
import { useAppDispatch, useAppSelector, setNavigation } from '@/store'
import Card from '@/components/common/Card'
import Button from '@/components/common/Button'
import Badge from '@/components/common/Badge'

interface StepReviewProps {
  onBack: () => void
}

export const StepReview: React.FC<StepReviewProps> = ({ onBack }) => {
  const dispatch = useAppDispatch()
  const { project, agent, platforms } = useAppSelector((state) => state.onboarding)
  const [isLaunching, setIsLaunching] = useState(false)
  const [expandedSections, setExpandedSections] = useState({
    project: true,
    agent: true,
    platforms: true,
  })

  const toggleSection = (section: 'project' | 'agent' | 'platforms') => {
    setExpandedSections((prev) => ({
      ...prev,
      [section]: !prev[section],
    }))
  }

  const handleLaunch = async () => {
    setIsLaunching(true)
    // Simulate API call to save configuration
    setTimeout(() => {
      setIsLaunching(false)
      dispatch(setNavigation('config'))
    }, 1500)
  }

  const connectedPlatformCount = Object.values(platforms).filter((p) => p.connected).length

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">Review your setup</h2>
        <p className="text-gray-600 dark:text-gray-400">Everything looks good? Launch your agent and start automating!</p>
      </div>

      {/* Project Section */}
      <Card elevation="lifted">
        <button
          onClick={() => toggleSection('project')}
          className="w-full flex items-center justify-between hover:text-emerald-600 dark:hover:text-emerald-400"
        >
          <div className="flex items-center gap-3">
            <span className="text-xl">📁</span>
            <div className="text-left">
              <h3 className="font-semibold text-gray-900 dark:text-gray-100">Project</h3>
              <p className="text-sm text-gray-500 dark:text-gray-400">{project.name}</p>
            </div>
          </div>
          <svg
            className={clsx('w-5 h-5 transition-transform', {
              'transform rotate-180': expandedSections.project,
            })}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
          </svg>
        </button>

        {expandedSections.project && (
          <div className="mt-4 space-y-2 border-t border-gray-200 dark:border-gray-700 pt-4">
            <div>
              <p className="text-sm text-gray-600 dark:text-gray-400">Name</p>
              <p className="font-medium text-gray-900 dark:text-gray-100">{project.name}</p>
            </div>
            {project.description && (
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Description</p>
                <p className="font-medium text-gray-900 dark:text-gray-100">{project.description}</p>
              </div>
            )}
          </div>
        )}
      </Card>

      {/* Agent Section */}
      <Card elevation="lifted">
        <button
          onClick={() => toggleSection('agent')}
          className="w-full flex items-center justify-between hover:text-emerald-600 dark:hover:text-emerald-400"
        >
          <div className="flex items-center gap-3">
            <span className="text-xl">🤖</span>
            <div className="text-left">
              <h3 className="font-semibold text-gray-900 dark:text-gray-100">Agent</h3>
              <p className="text-sm text-gray-500 dark:text-gray-400">{agent.name}</p>
            </div>
          </div>
          <svg
            className={clsx('w-5 h-5 transition-transform', {
              'transform rotate-180': expandedSections.agent,
            })}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
          </svg>
        </button>

        {expandedSections.agent && (
          <div className="mt-4 space-y-3 border-t border-gray-200 dark:border-gray-700 pt-4">
            <div>
              <p className="text-sm text-gray-600 dark:text-gray-400">Name</p>
              <p className="font-medium text-gray-900 dark:text-gray-100">{agent.name}</p>
            </div>
            <div>
              <p className="text-sm text-gray-600 dark:text-gray-400">Model</p>
              <p className="font-medium text-gray-900 dark:text-gray-100 capitalize">{agent.model}</p>
            </div>
            <div>
              <p className="text-sm text-gray-600 dark:text-gray-400">Type</p>
              <Badge variant="info" size="sm">
                {agent.type}
              </Badge>
            </div>
            {agent.capabilities && agent.capabilities.length > 0 && (
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">Capabilities</p>
                <div className="flex flex-wrap gap-2">
                  {agent.capabilities.map((cap) => (
                    <Badge key={cap} variant="success" size="sm">
                      {cap}
                    </Badge>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </Card>

      {/* Platforms Section */}
      <Card elevation="lifted">
        <button
          onClick={() => toggleSection('platforms')}
          className="w-full flex items-center justify-between hover:text-emerald-600 dark:hover:text-emerald-400"
        >
          <div className="flex items-center gap-3">
            <span className="text-xl">🌐</span>
            <div className="text-left">
              <h3 className="font-semibold text-gray-900 dark:text-gray-100">Platforms</h3>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                {connectedPlatformCount} platform{connectedPlatformCount !== 1 ? 's' : ''} connected
              </p>
            </div>
          </div>
          <svg
            className={clsx('w-5 h-5 transition-transform', {
              'transform rotate-180': expandedSections.platforms,
            })}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
          </svg>
        </button>

        {expandedSections.platforms && (
          <div className="mt-4 space-y-2 border-t border-gray-200 dark:border-gray-700 pt-4">
            {connectedPlatformCount > 0 ? (
              Object.entries(platforms)
                .filter(([, p]) => p.connected)
                .map(([type, p]) => (
                  <div key={type} className="flex items-center justify-between p-2 bg-emerald-50 dark:bg-emerald-900/20 rounded">
                    <span className="text-sm font-medium text-gray-900 dark:text-gray-100">✓ {p.name}</span>
                    {p.username && <span className="text-xs text-gray-500 dark:text-gray-400">@{p.username}</span>}
                  </div>
                ))
            ) : (
              <p className="text-sm text-gray-500 dark:text-gray-400">No platforms connected (optional)</p>
            )}
          </div>
        )}
      </Card>

      <div className="flex justify-between gap-3 pt-4">
        <Button variant="secondary" onClick={onBack}>
          Back
        </Button>
        <Button onClick={handleLaunch} loading={isLaunching} disabled={isLaunching}>
          Launch Agent
        </Button>
      </div>
    </div>
  )
}

import clsx from 'clsx'

export default StepReview
