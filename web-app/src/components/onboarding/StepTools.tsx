import React, { useEffect, useState } from 'react'
import { useAppDispatch, useAppSelector, updateTools, discoverTools } from '@/store'
import { Tool, ToolCategory, RECOMMENDED_TOOLS } from '@/types/onboarding'
import Checkbox from '@/components/common/Checkbox'
import LoadingSpinner from '@/components/common/LoadingSpinner'
import Alert from '@/components/common/Alert'

/**
 * Group tools by category
 */
function groupByCategory(tools: Tool[]): Record<ToolCategory, Tool[]> {
  const grouped: Record<ToolCategory, Tool[]> = {
    kubectl: [],
    terraform: [],
    docker: [],
    git: [],
    aws: [],
    shell: [],
    custom: [],
    other: [],
  }

  tools.forEach((tool) => {
    const category = tool.category || 'other'
    if (category in grouped) {
      grouped[category].push(tool)
    } else {
      grouped.other.push(tool)
    }
  })

  return grouped
}

/**
 * Step 3: Tool Discovery and Selection
 * Shows auto-discovered tools and lets users enable/disable them
 */
export const StepTools: React.FC = () => {
  const dispatch = useAppDispatch()
  const selectedTools = useAppSelector((state) => state.onboarding.selectedTools)
  const loading = useAppSelector((state) => state.onboarding.loading)
  const error = useAppSelector((state) => state.onboarding.error)

  const [availableTools, setAvailableTools] = useState<Tool[]>([])
  const [discoveryError, setDiscoveryError] = useState<string | null>(null)
  const [isDiscovering, setIsDiscovering] = useState(true)
  const [expandedCategories, setExpandedCategories] = useState<Record<ToolCategory, boolean>>({
    kubectl: true,
    terraform: true,
    docker: true,
    git: true,
    aws: true,
    shell: false,
    custom: false,
    other: false,
  })

  // Auto-discover tools on mount
  useEffect(() => {
    const discoverAvailableTools = async () => {
      setIsDiscovering(true)
      setDiscoveryError(null)

      try {
        // Use mock data for now since backend discovery isn't fully implemented
        const mockTools: Tool[] = [
          // Kubernetes
          {
            id: 'kubectl',
            name: 'kubectl',
            version: '1.29.0',
            path: '/usr/local/bin/kubectl',
            available: true,
            enabled: true,
            category: 'kubectl',
            description: 'Kubernetes command-line tool',
          },
          // Terraform
          {
            id: 'terraform',
            name: 'terraform',
            version: '1.6.0',
            path: '/usr/local/bin/terraform',
            available: true,
            enabled: true,
            category: 'terraform',
            description: 'Infrastructure as Code tool',
          },
          // Docker
          {
            id: 'docker',
            name: 'docker',
            version: '24.0.0',
            path: '/usr/bin/docker',
            available: true,
            enabled: true,
            category: 'docker',
            description: 'Container platform',
          },
          // Git
          {
            id: 'git',
            name: 'git',
            version: '2.42.0',
            path: '/usr/bin/git',
            available: true,
            enabled: true,
            category: 'git',
            description: 'Version control system',
          },
          // AWS CLI
          {
            id: 'aws-cli',
            name: 'aws',
            version: '2.13.0',
            path: '/usr/local/bin/aws',
            available: true,
            enabled: true,
            category: 'aws',
            description: 'Amazon Web Services CLI',
          },
          // Helm
          {
            id: 'helm',
            name: 'helm',
            version: '3.12.0',
            path: '/usr/local/bin/helm',
            available: true,
            enabled: true,
            category: 'kubectl',
            description: 'Kubernetes package manager',
          },
          // Shell/Bash
          {
            id: 'shell',
            name: 'bash',
            version: '5.2.0',
            path: '/bin/bash',
            available: true,
            enabled: true,
            category: 'shell',
            description: 'Shell command execution',
          },
          // jq
          {
            id: 'jq',
            name: 'jq',
            version: '1.7.0',
            path: '/usr/bin/jq',
            available: true,
            enabled: false,
            category: 'shell',
            description: 'JSON command-line processor',
          },
          // yq
          {
            id: 'yq',
            name: 'yq',
            version: '4.34.0',
            path: '/usr/local/bin/yq',
            available: true,
            enabled: false,
            category: 'shell',
            description: 'YAML command-line processor',
          },
        ]

        setAvailableTools(mockTools)

        // Pre-select recommended tools
        const recommended = mockTools.filter((t) =>
          RECOMMENDED_TOOLS.includes(t.name)
        )
        dispatch(updateTools(recommended))
      } catch (err: any) {
        setDiscoveryError(err.message || 'Failed to discover tools')
      } finally {
        setIsDiscovering(false)
      }
    }

    discoverAvailableTools()
  }, [dispatch])

  const handleToolToggle = (tool: Tool) => {
    const isSelected = selectedTools.some((t) => t.id === tool.id)

    if (isSelected) {
      dispatch(updateTools(selectedTools.filter((t) => t.id !== tool.id)))
    } else {
      dispatch(updateTools([...selectedTools, tool]))
    }
  }

  const toggleCategory = (category: ToolCategory) => {
    setExpandedCategories((prev) => ({
      ...prev,
      [category]: !prev[category],
    }))
  }

  const groupedTools = groupByCategory(availableTools)
  const visibleCategories = (Object.keys(groupedTools) as ToolCategory[]).filter(
    (cat) => groupedTools[cat].length > 0
  )

  if (isDiscovering) {
    return <LoadingSpinner text="Scanning for available tools..." />
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
          Which tools can Xops use?
        </h2>
        <p className="text-gray-600 dark:text-gray-400">
          Select the tools you want Xops to have access to. We found these on your system.
        </p>
      </div>

      {discoveryError && (
        <Alert variant="warning" title="Tool Discovery">
          Failed to auto-discover tools. You can add them manually later.
        </Alert>
      )}

      {/* Tool Categories */}
      <div className="space-y-4">
        {visibleCategories.map((category) => {
          const tools = groupedTools[category]
          const isExpanded = expandedCategories[category]
          const categoryToolsSelected = tools.filter((t) =>
            selectedTools.some((st) => st.id === t.id)
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
                  <span className="font-semibold text-gray-900 dark:text-white capitalize">
                    {category}
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
                    const isSelected = selectedTools.some((t) => t.id === tool.id)

                    return (
                      <label
                        key={tool.id}
                        className="flex items-start p-3 rounded hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition"
                      >
                        <Checkbox
                          checked={isSelected}
                          onChange={() => handleToolToggle(tool)}
                          disabled={!tool.available}
                        />
                        <div className="ml-3 flex-1">
                          <div className="font-medium text-gray-900 dark:text-white">
                            {tool.name}
                          </div>
                          {tool.description && (
                            <div className="text-sm text-gray-600 dark:text-gray-400">
                              {tool.description}
                            </div>
                          )}
                          <div className="text-xs text-gray-500 dark:text-gray-500 mt-1">
                            {tool.available ? (
                              <span className="text-green-600 dark:text-green-400">
                                ✓ Found at {tool.path}
                              </span>
                            ) : (
                              <span className="text-gray-400">
                                ✗ Not installed (v{tool.version})
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
