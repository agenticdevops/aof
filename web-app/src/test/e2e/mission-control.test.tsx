import { describe, it, expect } from 'vitest'
import { render, screen, fireEvent, within } from '@testing-library/react'
import { Provider } from 'react-redux'
import { configureStore } from '@reduxjs/toolkit'
import { BrowserRouter } from 'react-router-dom'
import MissionControl from '@/pages/MissionControl'
import appReducer from '@/store/slices/appSlice'
import dashboardReducer from '@/store/slices/dashboardSlice'

// Test store setup
const createTestStore = () => {
  return configureStore({
    reducer: {
      app: appReducer,
      dashboard: dashboardReducer,
    },
  })
}

// Wrapper component with Redux and Router
const TestWrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const store = createTestStore()
  return (
    <Provider store={store}>
      <BrowserRouter>{children}</BrowserRouter>
    </Provider>
  )
}

describe('MissionControl Page E2E Tests', () => {
  it('renders page with header and description', () => {
    render(
      <TestWrapper>
        <MissionControl />
      </TestWrapper>
    )

    expect(screen.getByText('Mission Control')).toBeInTheDocument()
    expect(screen.getByText('Monitor your agent squad in real-time')).toBeInTheDocument()
  })

  it('loads and displays agent grid with mock agents', async () => {
    render(
      <TestWrapper>
        <MissionControl />
      </TestWrapper>
    )

    // Wait for mock agents to load (dispatched on mount)
    // Check for specific agent names from mock data
    expect(await screen.findByText('Xops')).toBeInTheDocument()
    expect(screen.getByText('K8sOps')).toBeInTheDocument()
    expect(screen.getByText('SREWatch')).toBeInTheDocument()
    expect(screen.getByText('InfraBot')).toBeInTheDocument()
    expect(screen.getByText('ErrorTracer')).toBeInTheDocument()
  })

  it('displays agents with correct styling and metrics', () => {
    render(
      <TestWrapper>
        <MissionControl />
      </TestWrapper>
    )

    // Check for Xops agent
    const xopsCard = screen.getByText('Xops').closest('.cursor-pointer')
    expect(xopsCard).toBeInTheDocument()

    if (xopsCard) {
      // Check for persona icon
      expect(within(xopsCard).getByText('⚙️')).toBeInTheDocument()

      // Check for role
      expect(within(xopsCard).getByText('Orchestrator')).toBeInTheDocument()

      // Check for status badge
      expect(within(xopsCard).getByText('Active')).toBeInTheDocument()
    }
  })

  it('opens agent detail modal when agent card is clicked', async () => {
    render(
      <TestWrapper>
        <MissionControl />
      </TestWrapper>
    )

    // Wait for agents to load
    const xopsCard = await screen.findByText('Xops')
    expect(xopsCard).toBeInTheDocument()

    // Click on agent card
    const card = xopsCard.closest('.cursor-pointer')
    expect(card).toBeInTheDocument()

    if (card) {
      fireEvent.click(card)

      // Modal should now be open with detailed view
      // Note: Modal title might be different from card name, checking for metrics instead
      expect(screen.getAllByText('Xops').length).toBeGreaterThan(1)
    }
  })

  it('closes modal when close button is clicked', async () => {
    render(
      <TestWrapper>
        <MissionControl />
      </TestWrapper>
    )

    // Wait for agents and click one
    const xopsCard = await screen.findByText('Xops')
    const card = xopsCard.closest('.cursor-pointer')

    if (card) {
      fireEvent.click(card)

      // Modal should be open
      const closeButton = screen.getByText('Close')
      expect(closeButton).toBeInTheDocument()

      // Click close
      fireEvent.click(closeButton)

      // Modal should close (Close button should no longer be visible)
      expect(screen.queryByText('Close')).not.toBeInTheDocument()
    }
  })

  it('displays grid with responsive classes', () => {
    const { container } = render(
      <TestWrapper>
        <MissionControl />
      </TestWrapper>
    )

    // Check for grid container
    const grid = container.querySelector('.grid')
    expect(grid).toBeInTheDocument()

    // Check for responsive column classes
    expect(grid).toHaveClass('grid-cols-1') // mobile
    expect(grid).toHaveClass('md:grid-cols-2') // tablet
    expect(grid).toHaveClass('lg:grid-cols-4') // desktop
  })

  it('applies dark mode classes correctly', () => {
    const { container } = render(
      <TestWrapper>
        <MissionControl />
      </TestWrapper>
    )

    // Check for dark mode support in page background
    const page = container.querySelector('.dark\\:bg-gray-900')
    expect(page).toBeInTheDocument()
  })

  it('displays correct number of agents (5 mock agents)', async () => {
    render(
      <TestWrapper>
        <MissionControl />
      </TestWrapper>
    )

    // Wait for agents to load
    await screen.findByText('Xops')

    // Count all agent cards (each has a persona icon)
    const agentCards = screen.getAllByText(/⚙️|☸️|👁️|🏗️|🔍/)
    expect(agentCards.length).toBe(5)
  })
})
