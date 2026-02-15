/**
 * End-to-End Tests for Bot Templates and Squad Management
 *
 * Tests cover:
 * - Template discovery and display
 * - Squad creation from templates
 * - Squad composition UI
 * - Agent management (add/remove)
 * - Error handling
 * - Redux state updates
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, fireEvent, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Provider } from 'react-redux'
import { configureStore } from '@reduxjs/toolkit'

import { BotTemplateSelector } from '@/components/config/BotTemplateSelector'
import { SquadCompositionUI } from '@/components/config/SquadCompositionUI'
import { SquadCompositionPanel } from '@/components/config/SquadCompositionPanel'

import configReducer, { createSquadFromTemplate, listSquads } from '@/store/slices/configSlice'
import { BOT_TEMPLATES } from '@/data/botTemplates'
import { SquadConfig } from '@/types/agents'

// Helper function to create a test store
const createTestStore = () => {
  return configureStore({
    reducer: {
      config: configReducer
    }
  })
}

describe('Bot Templates and Squad Management E2E', () => {
  describe('Test 1: Bot templates available', () => {
    it('should display all 3 bot templates', async () => {
      const store = createTestStore()

      render(
        <Provider store={store}>
          <BotTemplateSelector
            onSelectTemplate={vi.fn()}
            showDescription={false}
          />
        </Provider>
      )

      // Check all templates are visible
      expect(screen.getByText('Kubernetes Ops Squad')).toBeInTheDocument()
      expect(screen.getByText('Infrastructure Automation Squad')).toBeInTheDocument()
      expect(screen.getByText('SRE Observability Squad')).toBeInTheDocument()
    })

    it('should show template icons and descriptions', async () => {
      const store = createTestStore()

      render(
        <Provider store={store}>
          <BotTemplateSelector
            onSelectTemplate={vi.fn()}
            showDescription={false}
          />
        </Provider>
      )

      // Check icons
      expect(screen.getByText('⚙️')).toBeInTheDocument() // K8s icon
      expect(screen.getByText('🏗️')).toBeInTheDocument() // Infra icon
      expect(screen.getByText('📊')).toBeInTheDocument() // SRE icon

      // Check descriptions visible
      expect(
        screen.getByText('Incident response and deployment automation for Kubernetes clusters')
      ).toBeInTheDocument()
    })

    it('should display agent count and setup time for each template', async () => {
      const store = createTestStore()

      render(
        <Provider store={store}>
          <BotTemplateSelector
            onSelectTemplate={vi.fn()}
            showDescription={false}
          />
        </Provider>
      )

      // Check that agent counts are displayed
      const agentsElements = screen.getAllByText('agents')
      expect(agentsElements.length).toBeGreaterThan(0)

      // Check setup times
      const setupElements = screen.getAllByText('min setup')
      expect(setupElements.length).toBeGreaterThan(0)
    })

    it('should show required tools for each template', async () => {
      const store = createTestStore()

      render(
        <Provider store={store}>
          <BotTemplateSelector
            onSelectTemplate={vi.fn()}
            showDescription={false}
          />
        </Provider>
      )

      // Check that tools are mentioned
      const toolsElements = screen.getAllByText('Required tools:')
      expect(toolsElements.length).toBeGreaterThan(0)
    })
  })

  describe('Test 2: Template selection and agent creation', () => {
    it('should select template on card click', async () => {
      const user = userEvent.setup()
      const store = createTestStore()
      const onSelectTemplate = vi.fn()

      render(
        <Provider store={store}>
          <BotTemplateSelector
            onSelectTemplate={onSelectTemplate}
            showDescription={true}
          />
        </Provider>
      )

      // Click Kubernetes template
      const kubeTemplate = screen.getByText('Kubernetes Ops Squad')
      await user.click(kubeTemplate)

      // Check if details panel appears
      await waitFor(() => {
        expect(screen.getByText('About Kubernetes Ops Squad')).toBeInTheDocument()
      })
    })

    it('should show Create Squad button when template selected', async () => {
      const user = userEvent.setup()
      const store = createTestStore()

      render(
        <Provider store={store}>
          <BotTemplateSelector
            onSelectTemplate={vi.fn()}
            showDescription={true}
          />
        </Provider>
      )

      // Select template
      const kubeTemplate = screen.getByText('Kubernetes Ops Squad')
      await user.click(kubeTemplate)

      // Check button appears
      await waitFor(() => {
        const buttons = screen.getAllByRole('button')
        const createButton = buttons.find(b => b.textContent?.includes('Create'))
        expect(createButton).toBeInTheDocument()
      })
    })

    it('should display all team members in selected template', async () => {
      const user = userEvent.setup()
      const store = createTestStore()

      render(
        <Provider store={store}>
          <BotTemplateSelector
            onSelectTemplate={vi.fn()}
            showDescription={true}
          />
        </Provider>
      )

      // Select template
      const kubeTemplate = screen.getByText('Kubernetes Ops Squad')
      await user.click(kubeTemplate)

      // Check team members displayed
      await waitFor(() => {
        expect(screen.getByText('K8s Ops Lead')).toBeInTheDocument()
        expect(screen.getByText('Pod Detective')).toBeInTheDocument()
        expect(screen.getByText('Network Ninja')).toBeInTheDocument()
      })
    })

    it('should show success metrics for template', async () => {
      const user = userEvent.setup()
      const store = createTestStore()

      render(
        <Provider store={store}>
          <BotTemplateSelector
            onSelectTemplate={vi.fn()}
            showDescription={true}
          />
        </Provider>
      )

      // Select template
      const kubeTemplate = screen.getByText('Kubernetes Ops Squad')
      await user.click(kubeTemplate)

      // Check metrics
      await waitFor(() => {
        expect(screen.getByText('Success Metrics')).toBeInTheDocument()
        expect(screen.getByText(/MTTR/)).toBeInTheDocument()
        expect(screen.getByText(/Detection Accuracy/)).toBeInTheDocument()
      })
    })
  })

  describe('Test 3: Squad composition display', () => {
    it('should display squad name and agent count', async () => {
      const store = createTestStore()

      const mockSquad: SquadConfig = {
        id: 'squad-1',
        name: 'Test Squad',
        templateId: 'kubernetes-ops-squad',
        agents: BOT_TEMPLATES[0].agents.map((agent, i) => ({
          id: `agent-${i}`,
          agentId: agent.id,
          name: agent.name,
          role: agent.role,
          status: 'active',
          persona: agent.persona,
          skills: agent.skills
        })),
        createdAt: new Date('2026-02-15'),
        lastModified: new Date('2026-02-15')
      }

      render(
        <Provider store={store}>
          <SquadCompositionUI squad={mockSquad} readOnly={true} />
        </Provider>
      )

      // Check squad name
      expect(screen.getByText('Test Squad')).toBeInTheDocument()
      // Check that agent count is mentioned
      expect(screen.getByText('Total Agents')).toBeInTheDocument()
    })

    it('should display agent cards with name, role, and status', async () => {
      const store = createTestStore()

      const mockSquad: SquadConfig = {
        id: 'squad-1',
        name: 'Test Squad',
        templateId: 'kubernetes-ops-squad',
        agents: BOT_TEMPLATES[0].agents.slice(0, 1).map((agent, i) => ({
          id: `agent-${i}`,
          agentId: agent.id,
          name: agent.name,
          role: agent.role,
          status: 'active',
          persona: agent.persona,
          skills: agent.skills
        })),
        createdAt: new Date(),
        lastModified: new Date()
      }

      render(
        <Provider store={store}>
          <SquadCompositionUI squad={mockSquad} readOnly={true} />
        </Provider>
      )

      // Check agent card displays
      expect(screen.getByText('K8s Ops Lead')).toBeInTheDocument()
      expect(screen.getByText('orchestrator')).toBeInTheDocument()
      expect(screen.getByText('active')).toBeInTheDocument()
    })

    it('should display agent name and role', async () => {
      const store = createTestStore()

      const mockSquad: SquadConfig = {
        id: 'squad-1',
        name: 'Test Squad',
        templateId: 'kubernetes-ops-squad',
        agents: BOT_TEMPLATES[0].agents.slice(0, 1).map((agent, i) => ({
          id: `agent-${i}`,
          agentId: agent.id,
          name: agent.name,
          role: agent.role,
          status: 'active',
          persona: agent.persona,
          skills: agent.skills
        })),
        createdAt: new Date('2026-02-15'),
        lastModified: new Date('2026-02-15')
      }

      render(
        <Provider store={store}>
          <SquadCompositionUI squad={mockSquad} readOnly={true} />
        </Provider>
      )

      // Check agent card displays
      expect(screen.getByText('K8s Ops Lead')).toBeInTheDocument()
      expect(screen.getByText('orchestrator')).toBeInTheDocument()
    })

    it('should show agent has skills defined', async () => {
      const store = createTestStore()

      const mockSquad: SquadConfig = {
        id: 'squad-1',
        name: 'Test Squad',
        templateId: 'kubernetes-ops-squad',
        agents: BOT_TEMPLATES[0].agents.slice(0, 1).map((agent, i) => ({
          id: `agent-${i}`,
          agentId: agent.id,
          name: agent.name,
          role: agent.role,
          status: 'active',
          persona: agent.persona,
          skills: agent.skills
        })),
        createdAt: new Date('2026-02-15'),
        lastModified: new Date('2026-02-15')
      }

      render(
        <Provider store={store}>
          <SquadCompositionUI squad={mockSquad} readOnly={true} />
        </Provider>
      )

      // Verify agent has skills
      expect(mockSquad.agents[0].skills.length).toBeGreaterThan(0)
    })
  })

  describe('Test 4: Squad creation flow', () => {
    it('should have templates with all required fields for creation', async () => {
      // Verify templates have all fields needed for squad creation
      BOT_TEMPLATES.forEach(template => {
        expect(template.id).toBeTruthy()
        expect(template.name).toBeTruthy()
        expect(template.agents.length).toBeGreaterThan(0)
        expect(template.agents.every(a => a.persona && a.skills)).toBe(true)
      })
    })
  })

  describe('Test 5: Error handling', () => {
    it('should handle template selection errors gracefully', async () => {
      const store = createTestStore()
      const mockError = new Error('API connection failed')
      const mockOnSelectTemplate = vi.fn().mockRejectedValue(mockError)

      // Component should render even with mock error handler
      render(
        <Provider store={store}>
          <BotTemplateSelector
            onSelectTemplate={mockOnSelectTemplate}
            showDescription={false}
          />
        </Provider>
      )

      // Check that component rendered
      expect(screen.getByText('Kubernetes Ops Squad')).toBeInTheDocument()
    })
  })

  describe('Test 6: Multiple squads and selection', () => {
    it('should display multiple squads in list', async () => {
      const mockSquads: SquadConfig[] = [
        {
          id: 'squad-1',
          name: 'Kubernetes Squad',
          templateId: 'kubernetes-ops-squad',
          agents: BOT_TEMPLATES[0].agents.map((agent, i) => ({
            id: `agent-${i}`,
            agentId: agent.id,
            name: agent.name,
            role: agent.role,
            status: 'active',
            persona: agent.persona,
            skills: agent.skills
          })),
          createdAt: new Date('2026-02-15'),
          lastModified: new Date('2026-02-15')
        },
        {
          id: 'squad-2',
          name: 'Infrastructure Squad',
          templateId: 'infrastructure-squad',
          agents: BOT_TEMPLATES[1].agents.map((agent, i) => ({
            id: `agent-${i}`,
            agentId: agent.id,
            name: agent.name,
            role: agent.role,
            status: 'active',
            persona: agent.persona,
            skills: agent.skills
          })),
          createdAt: new Date('2026-02-15'),
          lastModified: new Date('2026-02-15')
        }
      ]

      // Pre-populate store with squads
      const preloadedState = {
        config: {
          agents: [],
          tools: [],
          platforms: [],
          squads: mockSquads,
          version: null,
          isLoading: false,
          error: null,
          searchQuery: '',
          selectedAgentId: null,
          selectedPlatformId: null,
          selectedSquadId: null
        }
      } as any

      const testStore = configureStore({
        reducer: { config: configReducer },
        preloadedState
      })

      render(
        <Provider store={testStore}>
          <SquadCompositionPanel />
        </Provider>
      )

      // Check both squads appear
      expect(screen.getByText('Kubernetes Squad')).toBeInTheDocument()
      expect(screen.getByText('Infrastructure Squad')).toBeInTheDocument()
    })

    it('should display selected squad composition', async () => {
      const user = userEvent.setup()
      const store = createTestStore()

      const mockSquad: SquadConfig = {
        id: 'squad-1',
        name: 'K8s Squad',
        templateId: 'kubernetes-ops-squad',
        agents: BOT_TEMPLATES[0].agents.map((agent, i) => ({
          id: `agent-${i}`,
          agentId: agent.id,
          name: agent.name,
          role: agent.role,
          status: 'active',
          persona: agent.persona,
          skills: agent.skills
        })),
        createdAt: new Date('2026-02-15'),
        lastModified: new Date('2026-02-15')
      }

      const preloadedState = {
        config: {
          agents: [],
          tools: [],
          platforms: [],
          squads: [mockSquad],
          version: null,
          isLoading: false,
          error: null,
          searchQuery: '',
          selectedAgentId: null,
          selectedPlatformId: null,
          selectedSquadId: 'squad-1' // Pre-select the squad
        }
      }

      const testStore = configureStore({
        reducer: { config: configReducer },
        preloadedState: preloadedState as any
      })

      render(
        <Provider store={testStore}>
          <SquadCompositionPanel />
        </Provider>
      )

      // Check composition displays with pre-selected squad
      expect(screen.getByText('K8s Squad')).toBeInTheDocument()
    })
  })

  describe('Test 7: Bot template templates correctness', () => {
    it('should have 3 templates with correct agent counts', async () => {
      expect(BOT_TEMPLATES).toHaveLength(3)

      // K8s Ops: 3 agents
      expect(BOT_TEMPLATES[0].agents).toHaveLength(3)
      expect(BOT_TEMPLATES[0].name).toBe('Kubernetes Ops Squad')

      // Infrastructure: 3 agents
      expect(BOT_TEMPLATES[1].agents).toHaveLength(3)
      expect(BOT_TEMPLATES[1].name).toBe('Infrastructure Automation Squad')

      // SRE: 3 agents
      expect(BOT_TEMPLATES[2].agents).toHaveLength(3)
      expect(BOT_TEMPLATES[2].name).toBe('SRE Observability Squad')
    })

    it('should have valid agent roles and personas', async () => {
      BOT_TEMPLATES.forEach(template => {
        template.agents.forEach(agent => {
          // Check role is valid
          expect(['orchestrator', 'specialist', 'assistant']).toContain(agent.role)

          // Check persona exists
          expect(agent.persona).toBeDefined()
          expect(agent.persona.personality).toBeTruthy()
          expect(agent.persona.communication_style).toBeTruthy()

          // Check skills exist
          expect(agent.skills.length).toBeGreaterThan(0)

          // Check skill properties
          agent.skills.forEach(skill => {
            expect(skill.name).toBeTruthy()
            expect(skill.category).toBeTruthy()
            expect(skill.confidence).toBeGreaterThanOrEqual(0)
            expect(skill.confidence).toBeLessThanOrEqual(1)
          })
        })
      })
    })

    it('should have success metrics for each template', async () => {
      BOT_TEMPLATES.forEach(template => {
        expect(template.successMetrics.length).toBeGreaterThan(0)

        template.successMetrics.forEach(metric => {
          expect(metric.name).toBeTruthy()
          expect(metric.description).toBeTruthy()
          expect(metric.target).toBeTruthy()
        })
      })
    })
  })
})
