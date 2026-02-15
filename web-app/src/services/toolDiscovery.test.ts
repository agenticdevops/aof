/**
 * Tests for tool discovery service and utilities
 */

import { describe, it, expect } from 'vitest'
import {
  groupToolsByCategory,
  filterTools,
  getRecommendedTools,
  getCategoryIcon,
  formatCategoryName,
  isCriticalTool,
} from './toolDiscovery'
import type { Tool } from '@/api/config'

const mockTools: Tool[] = [
  {
    id: 'kubectl',
    name: 'Kubernetes CLI',
    path: '/usr/local/bin/kubectl',
    version: 'v1.29.0',
    category: 'Kubectl',
    available: true,
  },
  {
    id: 'terraform',
    name: 'Terraform',
    path: '/usr/local/bin/terraform',
    version: '1.6.0',
    category: 'Terraform',
    available: true,
  },
  {
    id: 'docker',
    name: 'Docker',
    path: '/usr/bin/docker',
    version: '25.0.0',
    category: 'Docker',
    available: true,
  },
  {
    id: 'vault',
    name: 'Vault',
    path: null,
    version: null,
    category: 'Custom',
    available: false,
  },
]

describe('groupToolsByCategory', () => {
  it('groups tools by category', () => {
    const grouped = groupToolsByCategory(mockTools)
    expect(Object.keys(grouped)).toContain('Kubectl')
    expect(Object.keys(grouped)).toContain('Terraform')
    expect(grouped['Kubectl']).toHaveLength(1)
    expect(grouped['Terraform']).toHaveLength(1)
  })

  it('handles empty tools array', () => {
    const grouped = groupToolsByCategory([])
    expect(Object.keys(grouped)).toHaveLength(0)
  })

  it('handles tools with no category', () => {
    const toolsNoCat = [{ id: 'test', name: 'Test' }] as Tool[]
    const grouped = groupToolsByCategory(toolsNoCat)
    expect(grouped['custom']).toBeDefined()
  })
})

describe('filterTools', () => {
  it('filters tools by id', () => {
    const filtered = filterTools(mockTools, 'kube')
    expect(filtered).toHaveLength(1)
    expect(filtered[0].id).toBe('kubectl')
  })

  it('filters tools by name', () => {
    const filtered = filterTools(mockTools, 'docker')
    expect(filtered).toHaveLength(1)
    expect(filtered[0].id).toBe('docker')
  })

  it('returns all tools for empty query', () => {
    const filtered = filterTools(mockTools, '')
    expect(filtered).toHaveLength(mockTools.length)
  })

  it('is case insensitive', () => {
    const filtered = filterTools(mockTools, 'KUBE')
    expect(filtered).toHaveLength(1)
    expect(filtered[0].id).toBe('kubectl')
  })

  it('returns empty array for no matches', () => {
    const filtered = filterTools(mockTools, 'nonexistent')
    expect(filtered).toHaveLength(0)
  })
})

describe('getRecommendedTools', () => {
  it('returns recommended tools', () => {
    const recommended = getRecommendedTools(mockTools)
    expect(recommended.length).toBeGreaterThan(0)
    expect(recommended.some(t => t.id === 'kubectl')).toBe(true)
    expect(recommended.some(t => t.id === 'terraform')).toBe(true)
  })

  it('only returns available tools', () => {
    const recommended = getRecommendedTools(mockTools)
    expect(recommended.every(t => t.available)).toBe(true)
  })

  it('returns empty array if no recommended tools available', () => {
    const unavailableTools = mockTools.map(t => ({ ...t, available: false }))
    const recommended = getRecommendedTools(unavailableTools)
    expect(recommended).toHaveLength(0)
  })
})

describe('getCategoryIcon', () => {
  it('returns correct icon for Kubectl', () => {
    const icon = getCategoryIcon('Kubectl')
    expect(icon).toBe('☸️')
  })

  it('returns correct icon for Docker', () => {
    const icon = getCategoryIcon('Docker')
    expect(icon).toBe('🐳')
  })

  it('returns default icon for unknown category', () => {
    const icon = getCategoryIcon('UnknownCategory')
    expect(icon).toBe('⚙️')
  })
})

describe('formatCategoryName', () => {
  it('formats category names correctly', () => {
    expect(formatCategoryName('Kubectl')).toBe('Kubectl')
    expect(formatCategoryName('DockerCompose')).toBe('Docker Compose')
  })

  it('handles single word categories', () => {
    expect(formatCategoryName('Custom')).toBe('Custom')
  })
})

describe('isCriticalTool', () => {
  it('identifies critical tools', () => {
    expect(isCriticalTool('kubectl')).toBe(true)
    expect(isCriticalTool('terraform')).toBe(true)
    expect(isCriticalTool('docker')).toBe(true)
    expect(isCriticalTool('git')).toBe(true)
    expect(isCriticalTool('aws')).toBe(true)
  })

  it('identifies non-critical tools', () => {
    expect(isCriticalTool('vault')).toBe(false)
    expect(isCriticalTool('prometheus')).toBe(false)
  })
})
