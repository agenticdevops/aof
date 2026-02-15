import React, { useState } from 'react'
import { useAppDispatch, useAppSelector, submitWizard, updateXopsConfig } from '@/store'
import { DEFAULT_XOPS_PERSONA } from '@/types/onboarding'
import Input from '@/components/common/Input'
import Badge from '@/components/common/Badge'
import Alert from '@/components/common/Alert'
import LoadingSpinner from '@/components/common/LoadingSpinner'

interface StepReviewProps {
  onBack: () => void
}

/**
 * Step 4: Review and Launch Xops
 * Shows Xops profile and launches the agent
 */
export const StepReview: React.FC<StepReviewProps> = ({ onBack }) => {
  const dispatch = useAppDispatch()
  const { selectedChannels, selectedModel, selectedTools, xopsConfig, loading, error } =
    useAppSelector((state) => state.onboarding)

  const [xopsName, setXopsName] = useState('Xops')
  const [submitted, setSubmitted] = useState(false)

  const handleLaunch = async () => {
    // Update config with final name
    dispatch(
      updateXopsConfig({
        ...xopsConfig,
        name: xopsName,
      })
    )

    // Submit wizard
    const result = await dispatch(submitWizard())

    if (submitWizard.fulfilled.match(result)) {
      setSubmitted(true)
      // Navigate to dashboard after a brief delay
      setTimeout(() => {
        window.location.hash = '#/dashboard'
      }, 1500)
    }
  }

  if (submitted) {
    return (
      <div className="text-center py-12">
        <div className="text-6xl mb-4">🎉</div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
          Welcome to Xops!
        </h2>
        <p className="text-gray-600 dark:text-gray-400 mb-8">
          Your orchestrator agent has been successfully created and is ready to coordinate.
        </p>

        <div className="inline-block p-6 bg-gradient-to-br from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 rounded-lg border border-blue-200 dark:border-blue-800">
          <div className="text-6xl mb-2">{DEFAULT_XOPS_PERSONA.emoji}</div>
          <h3 className="text-lg font-bold text-gray-900 dark:text-white">{xopsName}</h3>
          <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
            {DEFAULT_XOPS_PERSONA.description}
          </p>
        </div>

        <p className="mt-8 text-sm text-gray-600 dark:text-gray-400">
          Redirecting to dashboard...
        </p>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
          Launch Xops
        </h2>
        <p className="text-gray-600 dark:text-gray-400">
          Review your configuration and launch your orchestrator agent
        </p>
      </div>

      {error && (
        <Alert variant="error" title="Launch Failed">
          {error}
        </Alert>
      )}

      {loading && <LoadingSpinner text="Creating your Xops agent..." />}

      {!loading && (
        <>
          {/* Xops Profile Card */}
          <div className="border border-gray-200 dark:border-gray-700 rounded-lg p-6 bg-gradient-to-br from-gray-50 to-white dark:from-gray-800 dark:to-gray-900">
            {/* Header */}
            <div className="flex items-start gap-4 mb-6">
              <div className="text-6xl">{DEFAULT_XOPS_PERSONA.emoji}</div>
              <div className="flex-1">
                <Input
                  label="Agent Name"
                  placeholder="Xops"
                  value={xopsName}
                  onChange={(e) => setXopsName(e.target.value)}
                  fullWidth
                />
                <div className="mt-2 flex gap-2">
                  <Badge variant="info">Orchestrator</Badge>
                </div>
              </div>
            </div>

            {/* Persona Card */}
            <div className="border-t border-gray-200 dark:border-gray-700 pt-6 mb-6">
              <h3 className="font-semibold text-gray-900 dark:text-white mb-2">
                {DEFAULT_XOPS_PERSONA.name}
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">
                {DEFAULT_XOPS_PERSONA.description}
              </p>

              <div className="space-y-2">
                <div>
                  <span className="text-xs font-semibold text-gray-700 dark:text-gray-300">
                    Communication Style
                  </span>
                  <p className="text-sm text-gray-600 dark:text-gray-400">
                    {DEFAULT_XOPS_PERSONA.communication_style}
                  </p>
                </div>

                <div>
                  <span className="text-xs font-semibold text-gray-700 dark:text-gray-300">
                    Traits
                  </span>
                  <div className="flex flex-wrap gap-2 mt-1">
                    {DEFAULT_XOPS_PERSONA.traits.map((trait) => (
                      <Badge key={trait} variant="success">
                        {trait}
                      </Badge>
                    ))}
                  </div>
                </div>
              </div>
            </div>

            {/* Configuration Summary */}
            <div className="border-t border-gray-200 dark:border-gray-700 pt-6">
              <h3 className="font-semibold text-gray-900 dark:text-white mb-4">
                Configuration Summary
              </h3>

              <div className="space-y-4">
                {/* Channels */}
                <div>
                  <span className="text-xs font-semibold text-gray-700 dark:text-gray-300">
                    Connected Channels
                  </span>
                  <div className="flex flex-wrap gap-2 mt-2">
                    {selectedChannels.length > 0 ? (
                      selectedChannels.map((channel) => (
                        <Badge key={channel} variant="info">
                          {channel.charAt(0).toUpperCase() + channel.slice(1)}
                        </Badge>
                      ))
                    ) : (
                      <span className="text-sm text-gray-600 dark:text-gray-400">
                        No channels selected
                      </span>
                    )}
                  </div>
                </div>

                {/* AI Model */}
                <div>
                  <span className="text-xs font-semibold text-gray-700 dark:text-gray-300">
                    AI Model
                  </span>
                  <div className="mt-2">
                    <Badge variant="success">
                      {selectedModel ? selectedModel.toUpperCase() : 'Not selected'}
                    </Badge>
                  </div>
                </div>

                {/* Tools */}
                <div>
                  <span className="text-xs font-semibold text-gray-700 dark:text-gray-300">
                    Selected Tools ({selectedTools.length})
                  </span>
                  {selectedTools.length > 0 ? (
                    <div className="flex flex-wrap gap-2 mt-2">
                      {selectedTools.slice(0, 5).map((tool) => (
                        <Badge key={tool.id} variant="neutral">
                          {tool.name}
                        </Badge>
                      ))}
                      {selectedTools.length > 5 && (
                        <Badge variant="neutral">+{selectedTools.length - 5} more</Badge>
                      )}
                    </div>
                  ) : (
                    <span className="text-sm text-gray-600 dark:text-gray-400">
                      No tools selected (can be added later)
                    </span>
                  )}
                </div>
              </div>
            </div>
          </div>

          {/* Info Alert */}
          <Alert variant="info" title="You can customize Xops later">
            This is just the starting configuration. You can modify channels, tools, and
            behavior anytime from the configuration dashboard.
          </Alert>

          {/* Launch Button */}
          <div className="flex gap-2 pt-4">
            <button
              onClick={onBack}
              disabled={loading}
              className="flex-1 px-4 py-2 text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 transition"
            >
              Back
            </button>
            <button
              onClick={handleLaunch}
              disabled={loading || selectedChannels.length === 0}
              className="flex-1 px-4 py-3 text-white bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700 dark:from-blue-700 dark:to-indigo-700 dark:hover:from-blue-800 dark:hover:to-indigo-800 rounded font-semibold disabled:opacity-50 disabled:cursor-not-allowed transition"
            >
              {loading ? 'Launching...' : 'Launch Xops'}
            </button>
          </div>
        </>
      )}
    </div>
  )
}

export default StepReview
