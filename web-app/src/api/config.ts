import client from './client'
import type { Agent, Tool, Platform } from '@/types'
import type { BotTemplate, SquadConfig, SquadAgent } from '@/types/agents'
import { BOT_TEMPLATES } from '@/data/botTemplates'

export type { Tool }

export const configAPI = {
  // Agents
  getAgents: () => client.get<Agent[]>('/api/config/agents'),
  createAgent: (data: Partial<Agent>) => client.post<Agent>('/api/config/agents', data),
  updateAgent: (id: string, data: Partial<Agent>) =>
    client.put<Agent>(`/api/config/agents/${id}`, data),
  deleteAgent: (id: string) => client.delete(`/api/config/agents/${id}`),

  /**
   * Create agents from a bot template
   * Creates all agents defined in the template
   */
  createAgentFromTemplate: async (template: BotTemplate): Promise<SquadAgent[]> => {
    try {
      const agents: SquadAgent[] = []

      for (const templateAgent of template.agents) {
        const agentData = {
          name: templateAgent.name,
          role: templateAgent.role,
          persona: templateAgent.persona,
          skills: templateAgent.skills,
          status: 'active' as const
        }

        const response = await client.post<SquadAgent>(
          '/api/config/agents',
          agentData
        )
        agents.push(response.data)
      }

      return agents
    } catch (error) {
      if (error && typeof error === 'object' && 'response' in error) {
        const err = error as any
        throw new Error(`Failed to create agent squad: ${err.response?.data?.error}`)
      }
      throw error
    }
  },

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

  // Bot Templates
  listAvailableTemplates: async (): Promise<BotTemplate[]> => {
    // Return local templates (can be backend-driven in future)
    return BOT_TEMPLATES
  },

  getTemplate: async (templateId: string): Promise<BotTemplate | undefined> => {
    const templates = await configAPI.listAvailableTemplates()
    return templates.find(t => t.id === templateId)
  },

  // Squad Management
  listSquads: async (): Promise<SquadConfig[]> => {
    try {
      const response = await client.get<SquadConfig[]>('/api/config/squads')
      return response.data.map(squad => ({
        ...squad,
        createdAt: new Date(squad.createdAt as any),
        lastModified: new Date(squad.lastModified as any)
      }))
    } catch (error) {
      // Return empty list if endpoint doesn't exist yet
      return []
    }
  },

  getSquadConfig: async (squadId: string): Promise<SquadConfig> => {
    try {
      const response = await client.get<any>(`/api/config/squads/${squadId}`)
      return {
        ...response.data,
        createdAt: new Date(response.data.createdAt),
        lastModified: new Date(response.data.lastModified)
      }
    } catch (error) {
      throw new Error('Failed to load squad configuration')
    }
  },

  updateSquadConfig: async (
    squadId: string,
    updates: Partial<SquadConfig>
  ): Promise<SquadConfig> => {
    try {
      const response = await client.put<any>(`/api/config/squads/${squadId}`, updates)
      return {
        ...response.data,
        createdAt: new Date(response.data.createdAt),
        lastModified: new Date(response.data.lastModified)
      }
    } catch (error) {
      throw new Error('Failed to update squad configuration')
    }
  },

  deleteAgentFromSquad: async (squadId: string, agentId: string): Promise<void> => {
    try {
      await client.delete(`/api/config/squads/${squadId}/agents/${agentId}`)
    } catch (error) {
      throw new Error('Failed to remove agent from squad')
    }
  },

  deleteSquad: async (squadId: string): Promise<void> => {
    try {
      await client.delete(`/api/config/squads/${squadId}`)
    } catch (error) {
      throw new Error('Failed to delete squad')
    }
  }
}
