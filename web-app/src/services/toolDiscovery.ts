/**
 * Tool Discovery Service
 *
 * Provides hooks and utilities for discovering and managing available tools
 * on the system. Tools are fetched from the backend and cached locally.
 */

import { useEffect, useState, useCallback } from 'react'
import { configAPI, Tool } from '@/api/config'

export interface ToolDiscoveryState {
  tools: Tool[]
  loading: boolean
  error: string | null
  lastDiscoveryTime: Date | null
  retrying: boolean
  discover: () => Promise<void>
}

/**
 * Hook for discovering tools on the system
 *
 * Fetches available tools from the backend and provides loading/error states.
 * Tools are cached to avoid repeated API calls.
 */
export const useToolDiscovery = (): ToolDiscoveryState => {
  const [tools, setTools] = useState<Tool[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [lastDiscoveryTime, setLastDiscoveryTime] = useState<Date | null>(null)
  const [retrying, setRetrying] = useState(false)

  const discover = useCallback(async () => {
    if (loading) return // Prevent concurrent requests

    setLoading(true)
    setError(null)
    const startTime = Date.now()

    try {
      const discoveredTools = await configAPI.discoverTools()
      setTools(discoveredTools)
      setLastDiscoveryTime(new Date())

      // Log discovery summary
      const availableCount = discoveredTools.filter(t => t.available).length
      console.log(
        `Discovered ${availableCount}/${discoveredTools.length} tools in ${Date.now() - startTime}ms`
      )
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Unknown error'
      setError(message)
      console.error('Tool discovery failed:', message)
    } finally {
      setLoading(false)
      setRetrying(false)
    }
  }, [loading])

  // Discover tools on mount
  useEffect(() => {
    discover()
  }, [discover])

  return {
    tools,
    loading,
    error,
    lastDiscoveryTime,
    retrying,
    discover,
  }
}

/**
 * Group tools by category for display
 */
export const groupToolsByCategory = (tools: Tool[]): Record<string, Tool[]> => {
  const grouped: Record<string, Tool[]> = {}
  tools.forEach(tool => {
    const category = tool.category ?? 'custom'
    if (!grouped[category]) {
      grouped[category] = []
    }
    grouped[category].push(tool)
  })
  return grouped
}

/**
 * Filter tools by search query
 */
export const filterTools = (tools: Tool[], query: string): Tool[] => {
  const q = query.toLowerCase()
  return tools.filter(
    t => t.id.toLowerCase().includes(q) || t.name.toLowerCase().includes(q)
  )
}

/**
 * Get recommended tools for initial selection
 * Recommended tools are those that are critical for most DevOps workflows
 */
export const getRecommendedTools = (tools: Tool[]): Tool[] => {
  const recommended = ['kubectl', 'terraform', 'docker', 'git', 'aws', 'helm']
  return tools.filter(t => recommended.includes(t.id) && (t.available ?? false))
}

/**
 * Get tool icon/emoji by category
 */
export const getCategoryIcon = (category: string): string => {
  const icons: Record<string, string> = {
    Kubectl: '☸️',
    Terraform: '🏗️',
    Docker: '🐳',
    Git: '📦',
    AWS: '☁️',
    Shell: '💻',
    Helm: '⎈',
    Prometheus: '📊',
    GCloud: '☁️',
    Azure: '☁️',
    Custom: '⚙️',
  }
  return icons[category] || '⚙️'
}

/**
 * Format a tool category name for display
 */
export const formatCategoryName = (category: string): string => {
  return category
    .replace(/([A-Z])/g, ' $1') // Add space before capitals
    .trim()
}

/**
 * Check if a tool is in the critical tools list
 */
export const isCriticalTool = (toolId: string): boolean => {
  const critical = ['kubectl', 'terraform', 'docker', 'git', 'aws']
  return critical.includes(toolId)
}
