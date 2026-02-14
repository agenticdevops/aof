/**
 * Component tests for AgentCard persona display (Phase 5-04).
 *
 * Tests cover:
 * 1. Avatar emoji rendering
 * 2. Personality traits badge display
 * 3. Capabilities expand/collapse
 * 4. CAN/CANNOT color coding
 * 5. Introduction toast appearance
 * 6. Reliability metrics display
 * 7. Responsive layout (AgentGrid)
 * 8. Skill tags with truncation
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Provider } from 'react-redux';
import { configureStore } from '@reduxjs/toolkit';
import { AgentCard } from '../AgentCard';
import { AgentGrid } from '../AgentGrid';
import { PersonalityTraits } from '../PersonalityTraits';
import { CapabilityBoundaries } from '../CapabilityBoundaries';
import configReducer, { addIntroduction } from '../../store/configSlice';
import eventsReducer from '../../store/eventsSlice';
import type { Agent } from '../../types/events';

/**
 * Mock agent data for testing.
 */
const mockAgent: Agent = {
  id: 'k8s-monitor',
  name: 'Kubernetes Monitor',
  role: 'Infrastructure Specialist',
  avatar: '🤖',
  personality: 'Vigilant and detail-oriented',
  personality_traits: ['methodical', 'proactive', 'detail-oriented'],
  can: ['kubectl operations', 'pod debugging', 'log analysis', 'alerting'],
  cannot: ['modify cluster RBAC', 'delete PVs without approval'],
  skills: ['kubectl', 'jq', 'prometheus', 'grafana', 'helm'],
  status: 'idle',
  communication_style: 'calm-professional',
  tone: 'formal',
  intro_message: "I'm Kubernetes Monitor, your infrastructure specialist.",
  uptime_percent: 98,
  success_rate: 96,
};

/**
 * Create a minimal Redux store for testing.
 */
function createTestStore() {
  return configureStore({
    reducer: {
      config: configReducer,
      events: eventsReducer,
    },
  });
}

/**
 * Render helper with Redux Provider.
 */
function renderWithStore(
  ui: React.ReactElement,
  store = createTestStore(),
) {
  return {
    ...render(<Provider store={store}>{ui}</Provider>),
    store,
  };
}

// Mock fetch for useAgentsConfig
beforeEach(() => {
  vi.restoreAllMocks();
  global.fetch = vi.fn().mockImplementation((url: string) => {
    if (url === '/api/config/agents') {
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve([mockAgent]),
        headers: new Headers({ 'X-Config-Version': 'abc123' }),
      });
    }
    if (url === '/api/config/version') {
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve({ version: 'abc123' }),
      });
    }
    return Promise.resolve({ ok: false, status: 404 });
  }) as ReturnType<typeof vi.fn>;
});

describe('AgentCard', () => {
  // Test 1: Avatar emoji rendering
  it('renders avatar emoji from agent config', () => {
    renderWithStore(<AgentCard agent={mockAgent} />);

    // The avatar text should be visible in the card
    expect(screen.getByText('🤖')).toBeInTheDocument();
  });

  // Test 1b: Default avatar when none provided
  it('renders default avatar when agent has no avatar', () => {
    const agentNoAvatar: Agent = {
      ...mockAgent,
      avatar: undefined,
      role: 'monitor',
    };

    renderWithStore(<AgentCard agent={agentNoAvatar} />);

    // Should render the monitor default emoji
    expect(screen.getByText('👁️')).toBeInTheDocument();
  });

  // Test 2: Personality traits display as badges
  it('renders personality traits as badges', () => {
    renderWithStore(<AgentCard agent={mockAgent} />);

    expect(screen.getByText('methodical')).toBeInTheDocument();
    expect(screen.getByText('proactive')).toBeInTheDocument();
    expect(screen.getByText('detail-oriented')).toBeInTheDocument();
  });

  // Test 3: Capabilities section expands and collapses
  it('expands and collapses capabilities section', async () => {
    const user = userEvent.setup();

    renderWithStore(<AgentCard agent={mockAgent} />);

    // Capabilities header should be visible
    const capabilitiesButton = screen.getByText('Capabilities');
    expect(capabilitiesButton).toBeInTheDocument();

    // Initially collapsed - CAN/CANNOT items not visible
    expect(screen.queryByText('I CAN:')).not.toBeInTheDocument();

    // Click to expand
    await user.click(capabilitiesButton);

    // Now CAN/CANNOT sections should be visible
    expect(screen.getByText('I CAN:')).toBeInTheDocument();
    expect(screen.getByText('I CANNOT:')).toBeInTheDocument();

    // Click to collapse
    await user.click(capabilitiesButton);

    // CAN/CANNOT should be hidden again
    expect(screen.queryByText('I CAN:')).not.toBeInTheDocument();
  });

  // Test 4: CAN items in green, CANNOT items in red
  it('displays CAN section with green styling and CANNOT with red', async () => {
    const user = userEvent.setup();

    renderWithStore(<AgentCard agent={mockAgent} />);

    // Expand capabilities
    await user.click(screen.getByText('Capabilities'));

    // Check CAN header has green color class
    const canHeader = screen.getByText('I CAN:');
    expect(canHeader).toHaveClass('text-green-700');

    // Check CANNOT header has red color class
    const cannotHeader = screen.getByText('I CANNOT:');
    expect(cannotHeader).toHaveClass('text-red-700');
  });

  // Test 6: Reliability metrics display
  it('displays uptime and success rate metrics', () => {
    renderWithStore(<AgentCard agent={mockAgent} />);

    // Check for metric text
    expect(screen.getByText('Uptime 98%')).toBeInTheDocument();
    expect(screen.getByText('Success 96%')).toBeInTheDocument();
  });

  // Test 6b: Missing metrics show placeholder
  it('shows placeholder when metrics are unavailable', () => {
    const agentNoMetrics: Agent = {
      ...mockAgent,
      uptime_percent: undefined,
      success_rate: undefined,
    };

    renderWithStore(<AgentCard agent={agentNoMetrics} />);

    expect(screen.getByText('Uptime --')).toBeInTheDocument();
    expect(screen.getByText('Success --')).toBeInTheDocument();
  });

  // Test 8: Skill tags display with truncation
  it('displays skill tags truncated to 3 with +N more', () => {
    renderWithStore(<AgentCard agent={mockAgent} />);

    // First 3 skills visible
    expect(screen.getByText('kubectl')).toBeInTheDocument();
    expect(screen.getByText('jq')).toBeInTheDocument();
    expect(screen.getByText('prometheus')).toBeInTheDocument();

    // +2 more indicator
    expect(screen.getByText('+2')).toBeInTheDocument();

    // 4th and 5th skills NOT directly visible
    expect(screen.queryByText('grafana')).not.toBeInTheDocument();
    expect(screen.queryByText('helm')).not.toBeInTheDocument();
  });

  // Test: Agent card is keyboard accessible
  it('is keyboard accessible with proper aria attributes', () => {
    renderWithStore(<AgentCard agent={mockAgent} />);

    const card = screen.getByRole('button', {
      name: /Agent Kubernetes Monitor/i,
    });
    expect(card).toHaveAttribute('tabindex', '0');
    expect(card).toHaveAttribute(
      'aria-label',
      'Agent Kubernetes Monitor, role: Infrastructure Specialist, status: idle',
    );
  });

  // Test: Click handler fires
  it('calls onClick with agent id when clicked', async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();

    renderWithStore(<AgentCard agent={mockAgent} onClick={onClick} />);

    const card = screen.getByRole('button', {
      name: /Agent Kubernetes Monitor/i,
    });
    await user.click(card);

    expect(onClick).toHaveBeenCalledWith('k8s-monitor');
  });
});

describe('PersonalityTraits', () => {
  it('renders nothing when traits array is empty', () => {
    const { container } = render(<PersonalityTraits traits={[]} />);
    expect(container.firstChild).toBeNull();
  });

  it('renders single trait as badge', () => {
    render(<PersonalityTraits traits={['curious']} />);
    expect(screen.getByText('curious')).toBeInTheDocument();
  });

  it('renders exactly 3 traits without more link', () => {
    render(
      <PersonalityTraits traits={['methodical', 'proactive', 'detail-oriented']} />,
    );

    expect(screen.getByText('methodical')).toBeInTheDocument();
    expect(screen.getByText('proactive')).toBeInTheDocument();
    expect(screen.getByText('detail-oriented')).toBeInTheDocument();
    expect(screen.queryByText(/more/)).not.toBeInTheDocument();
  });

  it('shows +N more when more than 3 traits', () => {
    render(
      <PersonalityTraits
        traits={['methodical', 'proactive', 'detail-oriented', 'calm', 'decisive']}
      />,
    );

    expect(screen.getByText('methodical')).toBeInTheDocument();
    expect(screen.getByText('proactive')).toBeInTheDocument();
    expect(screen.getByText('detail-oriented')).toBeInTheDocument();
    expect(screen.getByText('+2 more')).toBeInTheDocument();
  });

  it('shows tooltip on hover', async () => {
    const user = userEvent.setup();

    render(<PersonalityTraits traits={['curious']} />);

    const badge = screen.getByRole('listitem', { name: /Trait: curious/ });
    await user.hover(badge);

    expect(screen.getByRole('tooltip')).toHaveTextContent(
      'This agent is curious',
    );
  });
});

describe('CapabilityBoundaries', () => {
  it('renders nothing when both arrays are empty', () => {
    const { container } = render(
      <CapabilityBoundaries can={[]} cannot={[]} />,
    );
    expect(container.firstChild).toBeNull();
  });

  it('renders header when either array has items', () => {
    render(
      <CapabilityBoundaries can={['kubectl']} cannot={[]} />,
    );
    expect(screen.getByText('Capabilities')).toBeInTheDocument();
  });

  it('toggles expand/collapse on click', async () => {
    const user = userEvent.setup();

    render(
      <CapabilityBoundaries
        can={['kubectl operations']}
        cannot={['modify RBAC']}
      />,
    );

    const header = screen.getByText('Capabilities');

    // Initially collapsed
    expect(header.closest('button')).toHaveAttribute('aria-expanded', 'false');

    // Expand
    await user.click(header);
    expect(header.closest('button')).toHaveAttribute('aria-expanded', 'true');
    expect(screen.getByText('kubectl operations')).toBeInTheDocument();
    expect(screen.getByText('modify RBAC')).toBeInTheDocument();

    // Collapse
    await user.click(header);
    expect(header.closest('button')).toHaveAttribute('aria-expanded', 'false');
    expect(screen.queryByText('kubectl operations')).not.toBeInTheDocument();
  });

  it('supports keyboard toggle with Enter key', async () => {
    render(
      <CapabilityBoundaries
        can={['kubectl operations']}
        cannot={['modify RBAC']}
      />,
    );

    const header = screen.getByText('Capabilities').closest('button')!;
    header.focus();

    // Press Enter
    fireEvent.keyDown(header, { key: 'Enter' });
    expect(header).toHaveAttribute('aria-expanded', 'true');

    // Press Space
    fireEvent.keyDown(header, { key: ' ' });
    expect(header).toHaveAttribute('aria-expanded', 'false');
  });
});

describe('AgentGrid responsive layout', () => {
  // Test 7: Grid uses responsive classes
  it('renders grid with responsive Tailwind classes', async () => {
    const store = createTestStore();

    renderWithStore(<AgentGrid />, store);

    // Wait for loading to complete
    const grid = await screen.findByRole('button', { name: /Agent Kubernetes Monitor/i });
    const gridContainer = grid.parentElement;

    expect(gridContainer).toHaveClass('grid');
    expect(gridContainer).toHaveClass('grid-cols-1');
    expect(gridContainer).toHaveClass('md:grid-cols-2');
    expect(gridContainer).toHaveClass('lg:grid-cols-3');
  });

  // Test 5: Introduction toast on event dispatch
  it('processes introduction events from Redux store', () => {
    const store = createTestStore();

    store.dispatch(
      addIntroduction({
        agent_name: 'Test Agent',
        intro_message: 'Hello, I am here to help!',
        skills: ['testing'],
        avatar: '🧪',
      }),
    );

    const state = store.getState();
    expect(state.config.introductions).toHaveLength(1);
    expect(state.config.introductions[0].intro_message).toBe(
      'Hello, I am here to help!',
    );
    expect(state.config.introducedAgentIds).toContain('Test Agent');
  });

  it('deduplicates introduction events for same agent', () => {
    const store = createTestStore();

    store.dispatch(
      addIntroduction({
        agent_name: 'Test Agent',
        intro_message: 'First introduction',
        skills: ['testing'],
        avatar: '🧪',
      }),
    );

    store.dispatch(
      addIntroduction({
        agent_name: 'Test Agent',
        intro_message: 'Second introduction',
        skills: ['testing'],
        avatar: '🧪',
      }),
    );

    const state = store.getState();
    expect(state.config.introductions).toHaveLength(1);
    expect(state.config.introductions[0].intro_message).toBe(
      'First introduction',
    );
  });
});
