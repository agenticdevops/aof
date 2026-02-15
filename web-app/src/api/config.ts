import client from './client'
import type { Agent, Tool, Platform } from '@/types'

export const configAPI = {
  // Agents
  getAgents: () => client.get<Agent[]>('/api/config/agents'),
  createAgent: (data: Partial<Agent>) => client.post<Agent>('/api/config/agents', data),
  updateAgent: (id: string, data: Partial<Agent>) =>
    client.put<Agent>(`/api/config/agents/${id}`, data),
  deleteAgent: (id: string) => client.delete(`/api/config/agents/${id}`),

  // Tools
  getTools: () => client.get<Tool[]>('/api/config/tools'),

  // Platforms
  getPlatforms: () => client.get<Platform[]>('/api/config/platforms'),
  testPlatform: (platform: string, config: any) =>
    client.post(`/api/config/platforms/${platform}/test`, config),

  // Version
  getVersion: () => client.get<{ version: string }>('/api/config/version'),
}
