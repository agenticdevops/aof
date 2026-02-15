import client from './client'
import type { Agent, Tool, Platform } from '@/types'

export type { Tool }

export const configAPI = {
  // Agents
  getAgents: () => client.get<Agent[]>('/api/config/agents'),
  createAgent: (data: Partial<Agent>) => client.post<Agent>('/api/config/agents', data),
  updateAgent: (id: string, data: Partial<Agent>) =>
    client.put<Agent>(`/api/config/agents/${id}`, data),
  deleteAgent: (id: string) => client.delete(`/api/config/agents/${id}`),

  // Tools
  getTools: () => client.get<Tool[]>('/api/config/tools'),

  /**
   * Discover available tools on the system
   * Scans system paths for installed DevOps tools
   */
  discoverTools: async (): Promise<Tool[]> => {
    try {
      const response = await client.get<Tool[]>('/api/config/tools/discover')
      return response.data
    } catch (error) {
      if (error && typeof error === 'object' && 'response' in error) {
        const err = error as any
        if (err.response?.status === 500) {
          throw new Error('Tool discovery failed: ' + err.response.data?.error)
        }
        throw new Error('Failed to discover tools: ' + err.message)
      }
      throw error
    }
  },

  // Optional: Get tools by category for filtering
  getToolsByCategory: async (category: string): Promise<Tool[]> => {
    const all = await configAPI.discoverTools()
    return all.filter(t => t.category === category)
  },

  // Optional: Check if specific tool is available
  isToolAvailable: async (toolId: string): Promise<boolean> => {
    const tools = await configAPI.discoverTools()
    return tools.some(t => t.id === toolId && t.available)
  },

  // Platforms
  getPlatforms: () => client.get<Platform[]>('/api/config/platforms'),
  testPlatform: (platform: string, config: any) =>
    client.post(`/api/config/platforms/${platform}/test`, config),

  // Version
  getVersion: () => client.get<{ version: string }>('/api/config/version'),
}
