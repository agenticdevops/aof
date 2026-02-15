import React, { useEffect, useState } from 'react'
import { useAppDispatch, useAppSelector, updateTools } from '@/store'
import { Tool as OnboardingTool, ToolCategory, RECOMMENDED_TOOLS } from '@/types/onboarding'
import Checkbox from '@/components/common/Checkbox'
import LoadingSpinner from '@/components/common/LoadingSpinner'
import Alert from '@/components/common/Alert'
import { useToolDiscovery, groupToolsByCategory, getRecommendedTools, formatCategoryName, getCategoryIcon } from '@/services/toolDiscovery'
import type { Tool as DiscoveredTool } from '@/api/config'

/**
 * Convert discovered tool to onboarding tool format
 */
function convertToOnboardingTool(tool: DiscoveredTool): OnboardingTool {
  return {
    id: tool.id,
    name: tool.name,
    version: tool.version ?? 'unknown',
    path: tool.path ?? '',
    available: tool.available ?? false,
    enabled: tool.available ?? false,
    category: (tool.category ?? 'custom').toLowerCase() as ToolCategory,
    description: `${tool.name}${tool.version ? ` (${tool.version})` : ''}`,
  }
}

/**
 * Step 3: Tool Discovery and Selection
 * Shows auto-discovered tools and lets users enable/disable them
 */
export const StepTools: React.FC = () => {
  const dispatch = useAppDispatch()
  const selectedTools = useAppSelector((state) => state.onboarding.selectedTools)

  // Use the real tool discovery hook
  const { tools: discoveredTools, loading, error, discover, lastDiscoveryTime } = useToolDiscovery()

  const [expandedCategories, setExpandedCategories] = useState<Record<string, boolean>>({})

  // Convert discovered tools to onboarding format
  const availableTools = discoveredTools.map(convertToOnboardingTool)

  // Initialize expanded categories and pre-select recommended tools on mount
  useEffect(() => {
    if (availableTools.length > 0) {
      // Initialize expansion state
      const grouped = groupToolsByCategory(discoveredTools)
      const initialExpanded: Record<string, boolean> = {}
      Object.keys(grouped).forEach(category => {
        initialExpanded[category] = true // Expand all by default
      })
      setExpandedCategories(initialExpanded)

      // Pre-select recommended tools
      const recommended = getRecommendedTools(discoveredTools)
      const recommendedOnboarding = recommended.map(convertToOnboardingTool)
      dispatch(updateTools(recommendedOnboarding))
    }
  }, [availableTools, dispatch, discoveredTools])

  const handleToolToggle = (tool: OnboardingTool) => {
    const isSelected = selectedTools.some((t) => t.id === tool.id)

    if (isSelected) {
      dispatch(updateTools(selectedTools.filter((t) => t.id !== tool.id)))
    } else {
      dispatch(updateTools([...selectedTools, tool]))
    }
  }

  const toggleCategory = (category: string) => {
    setExpandedCategories((prev) => ({
      ...prev,
      [category]: !prev[category],
    }))
  }

  const groupedTools = groupToolsByCategory(discoveredTools)
  const visibleCategories = Object.keys(groupedTools).filter(
    (cat) => groupedTools[cat].length > 0
  )

  if (loading) {
    return <LoadingSpinner text="Scanning for available tools..." />
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
          Which tools can Xops use?
        </h2>
        <p className="text-gray-600 dark:text-gray-400">
          Select the tools you want Xops to have access to. We found {availableTools.length} on your system.
          {lastDiscoveryTime && (
            <span className="text-sm text-gray-500">
              {' '}(discovered {lastDiscoveryTime.toLocaleTimeString()})
            </span>
          )}
        </p>
      </div>

      {error && (
        <Alert variant="warning" title="Tool Discovery Failed">
          {error}
          <button
            onClick={discover}
            className="ml-2 underline hover:no-underline font-medium"
          >
            Retry
          </button>
        </Alert>
      )}

      {/* Tool Categories */}
      {availableTools.length === 0 ? (
        <Alert variant="info" title="No tools found">
          No tools were discovered on your system. You can add tools manually in the configuration dashboard.
        </Alert>
      ) : (
        <div className="space-y-4">
          {visibleCategories.map((category) => {
            const tools = groupedTools[category]
            const isExpanded = expandedCategories[category]
            const categoryToolsSelected = availableTools.filter(
              (t) =>
                t.category === category && selectedTools.some((st) => st.id === t.id)
            ).length

            return (
              <div
                key={category}
                className="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden"
              >
                {/* Category Header */}
                <button
                  onClick={() => toggleCategory(category)}
                  className="w-full px-4 py-3 bg-gray-50 dark:bg-gray-800 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center justify-between transition"
                >
                  <div className="flex items-center gap-3 flex-1 text-left">
                    <span className={`transition ${isExpanded ? 'rotate-90' : ''}`}>▶</span>
                    <span className="text-lg">{getCategoryIcon(category)}</span>
                    <span className="font-semibold text-gray-900 dark:text-white capitalize">
                      {formatCategoryName(category)}
                    </span>
                    <span className="text-xs text-gray-500 dark:text-gray-400">
                      {categoryToolsSelected} of {tools.length} selected
                    </span>
                  </div>
                </button>

                {/* Category Tools */}
                {isExpanded && (
                  <div className="p-4 space-y-3 bg-white dark:bg-gray-900/50">
                    {tools.map((tool) => {
                      const onboardingTool = convertToOnboardingTool(tool)
                      const isSelected = selectedTools.some((t) => t.id === tool.id)

                      return (
                        <label
                          key={tool.id}
                          className="flex items-start p-3 rounded hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition"
                        >
                          <Checkbox
                            checked={isSelected}
                            onChange={() => handleToolToggle(onboardingTool)}
                            disabled={!tool.available}
                          />
                          <div className="ml-3 flex-1">
                            <div className="font-medium text-gray-900 dark:text-white">
                              {tool.name}
                            </div>
                            {tool.version && (
                              <div className="text-sm text-gray-600 dark:text-gray-400">
                                Version: {tool.version}
                              </div>
                            )}
                            <div className="text-xs text-gray-500 dark:text-gray-500 mt-1">
                              {tool.available ? (
                                <span className="text-green-600 dark:text-green-400">
                                  ✓ Found at {tool.path}
                                </span>
                              ) : (
                                <span className="text-gray-400">
                                  ✗ Not installed
                                </span>
                              )}
                            </div>
                          </div>
                        </label>
                      )
                    })}
                  </div>
                )}
              </div>
            )
          })}
        </div>
      )}

      {/* Summary */}
      <div className="border-t border-gray-200 dark:border-gray-700 pt-4">
        <div className="text-sm font-medium text-gray-900 dark:text-white mb-2">
          Selected tools: {selectedTools.length}
        </div>

        {selectedTools.length === 0 && (
          <Alert variant="info" title="No tools selected">
            Xops will have limited capabilities without tools. You can add them later in the
            configuration dashboard.
          </Alert>
        )}

        {selectedTools.length > 0 && (
          <div className="flex flex-wrap gap-2">
            {selectedTools.map((tool) => (
              <span
                key={tool.id}
                className="px-3 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-200 rounded-full text-sm"
              >
                {tool.name}
              </span>
            ))}
          </div>
        )}
      </div>

      {/* Recommended Tools Info */}
      <div className="p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg text-yellow-800 dark:text-yellow-200 text-sm">
        <strong>💡 Recommended:</strong> We pre-selected common tools like kubectl, terraform,
        and docker. Uncheck any you don't need.
      </div>
    </div>
  )
}

export default StepTools
