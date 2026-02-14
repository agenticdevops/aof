/**
 * HeartbeatDashboard component tests.
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { HeartbeatDashboard } from '../HeartbeatDashboard';
import type { AgentHealthRecord } from '../../types/coordination';

describe('HeartbeatDashboard', () => {
  it('renders empty state when coordination disabled', () => {
    render(
      <HeartbeatDashboard
        health={[]}
        coordinationEnabled={false}
      />
    );

    expect(screen.getByText(/coordination not enabled/i)).toBeInTheDocument();
    expect(screen.getByText(/enable coordination protocols/i)).toBeInTheDocument();
  });

  it('renders empty state when no agents', () => {
    render(
      <HeartbeatDashboard
        health={[]}
        coordinationEnabled={true}
      />
    );

    expect(screen.getByText(/coordination not enabled/i)).toBeInTheDocument();
  });

  it('renders agent health cards with correct status colors', () => {
    const health: AgentHealthRecord[] = [
      {
        agent_id: 'agent-1',
        status: 'Healthy',
        last_heartbeat: new Date().toISOString(),
        consecutive_misses: 0,
        last_response_ms: 100,
      },
      {
        agent_id: 'agent-2',
        status: 'Degraded',
        last_heartbeat: new Date().toISOString(),
        consecutive_misses: 0,
        last_response_ms: 500,
        degraded_reason: 'Slow response',
      },
      {
        agent_id: 'agent-3',
        status: 'Unresponsive',
        last_heartbeat: null,
        consecutive_misses: 3,
        last_response_ms: null,
      },
    ];

    const { container } = render(
      <HeartbeatDashboard
        health={health}
        coordinationEnabled={true}
      />
    );

    // Check summary bar
    expect(screen.getByText(/2\/3 healthy/i)).toBeInTheDocument();
    expect(screen.getByText(/1 degraded/i)).toBeInTheDocument();
    expect(screen.getByText(/1 unresponsive/i)).toBeInTheDocument();

    // Check agent cards exist
    expect(screen.getByText('agent-1')).toBeInTheDocument();
    expect(screen.getByText('agent-2')).toBeInTheDocument();
    expect(screen.getByText('agent-3')).toBeInTheDocument();

    // Check status text
    expect(screen.getAllByText('Healthy')).toHaveLength(1);
    expect(screen.getByText('Degraded')).toBeInTheDocument();
    expect(screen.getByText('Unresponsive')).toBeInTheDocument();

    // Check degraded reason
    expect(screen.getByText('Slow response')).toBeInTheDocument();

    // Check consecutive misses
    expect(screen.getByText(/missed: 3x/i)).toBeInTheDocument();

    // Check that cards have correct data attributes
    const unresponsiveCard = container.querySelector('[data-status="Unresponsive"]');
    expect(unresponsiveCard).toBeInTheDocument();
  });

  it('shows last heartbeat time', () => {
    const fiveSecondsAgo = new Date(Date.now() - 5000).toISOString();

    const health: AgentHealthRecord[] = [
      {
        agent_id: 'test-agent',
        status: 'Healthy',
        last_heartbeat: fiveSecondsAgo,
        consecutive_misses: 0,
        last_response_ms: 100,
      },
    ];

    render(
      <HeartbeatDashboard
        health={health}
        coordinationEnabled={true}
      />
    );

    // Should show relative time
    expect(screen.getByText(/5s ago/i)).toBeInTheDocument();
  });

  it('shows "never" for null last_heartbeat', () => {
    const health: AgentHealthRecord[] = [
      {
        agent_id: 'test-agent',
        status: 'Unresponsive',
        last_heartbeat: null,
        consecutive_misses: 5,
        last_response_ms: null,
      },
    ];

    render(
      <HeartbeatDashboard
        health={health}
        coordinationEnabled={true}
      />
    );

    expect(screen.getByText(/never/i)).toBeInTheDocument();
  });

  it('displays response latency when available', () => {
    const health: AgentHealthRecord[] = [
      {
        agent_id: 'fast-agent',
        status: 'Healthy',
        last_heartbeat: new Date().toISOString(),
        consecutive_misses: 0,
        last_response_ms: 123,
      },
    ];

    render(
      <HeartbeatDashboard
        health={health}
        coordinationEnabled={true}
      />
    );

    expect(screen.getByText(/latency: 123ms/i)).toBeInTheDocument();
  });

  it('renders correctly with only healthy agents', () => {
    const health: AgentHealthRecord[] = [
      {
        agent_id: 'agent-1',
        status: 'Healthy',
        last_heartbeat: new Date().toISOString(),
        consecutive_misses: 0,
        last_response_ms: 100,
      },
      {
        agent_id: 'agent-2',
        status: 'Healthy',
        last_heartbeat: new Date().toISOString(),
        consecutive_misses: 0,
        last_response_ms: 150,
      },
    ];

    render(
      <HeartbeatDashboard
        health={health}
        coordinationEnabled={true}
      />
    );

    // Summary should show all healthy
    expect(screen.getByText(/2\/2 healthy/i)).toBeInTheDocument();

    // Should NOT show degraded or unresponsive sections
    expect(screen.queryByText(/degraded/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/unresponsive/i)).not.toBeInTheDocument();
  });
});
