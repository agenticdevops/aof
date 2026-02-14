/**
 * CoordinationStatus component tests.
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { CoordinationStatus } from '../CoordinationStatus';
import type { CoordinationMetrics } from '../../types/coordination';

describe('CoordinationStatus', () => {
  it('renders "unavailable" message when no metrics', () => {
    const mockForceMode = vi.fn();

    render(
      <CoordinationStatus
        metrics={null}
        onForceMode={mockForceMode}
      />
    );

    expect(screen.getByText(/coordination metrics unavailable/i)).toBeInTheDocument();
  });

  it('displays current mode badge', () => {
    const mockForceMode = vi.fn();

    const metrics: CoordinationMetrics = {
      coordination_tokens: 10000,
      production_tokens: 100000,
      overhead_percent: 10,
      heartbeat_tokens: 5000,
      standup_tokens: 5000,
      current_mode: 'Standard',
      auto_degrade_enabled: true,
      max_overhead_percent: 30,
      window_start: new Date().toISOString(),
    };

    render(
      <CoordinationStatus
        metrics={metrics}
        onForceMode={mockForceMode}
      />
    );

    expect(screen.getByText('Standard')).toBeInTheDocument();
  });

  it('shows correct overhead color for green threshold (<20%)', () => {
    const mockForceMode = vi.fn();

    const metrics: CoordinationMetrics = {
      coordination_tokens: 10000,
      production_tokens: 100000,
      overhead_percent: 10,
      heartbeat_tokens: 5000,
      standup_tokens: 5000,
      current_mode: 'Full',
      auto_degrade_enabled: true,
      max_overhead_percent: 30,
      window_start: new Date().toISOString(),
    };

    const { container } = render(
      <CoordinationStatus
        metrics={metrics}
        onForceMode={mockForceMode}
      />
    );

    // Overhead should be green (< 20%)
    expect(screen.getByText(/10%/)).toBeInTheDocument();

    // Check that gauge bar has green color class
    const gaugeBar = container.querySelector('.bg-green-500');
    expect(gaugeBar).toBeInTheDocument();
  });

  it('shows correct overhead color for yellow threshold (20-30%)', () => {
    const mockForceMode = vi.fn();

    const metrics: CoordinationMetrics = {
      coordination_tokens: 25000,
      production_tokens: 100000,
      overhead_percent: 25,
      heartbeat_tokens: 12000,
      standup_tokens: 13000,
      current_mode: 'Standard',
      auto_degrade_enabled: true,
      max_overhead_percent: 30,
      window_start: new Date().toISOString(),
    };

    const { container } = render(
      <CoordinationStatus
        metrics={metrics}
        onForceMode={mockForceMode}
      />
    );

    expect(screen.getByText(/25%/)).toBeInTheDocument();

    // Check that gauge bar has yellow color class
    const gaugeBar = container.querySelector('.bg-yellow-500');
    expect(gaugeBar).toBeInTheDocument();
  });

  it('shows correct overhead color for red threshold (>30%)', () => {
    const mockForceMode = vi.fn();

    const metrics: CoordinationMetrics = {
      coordination_tokens: 40000,
      production_tokens: 100000,
      overhead_percent: 40,
      heartbeat_tokens: 20000,
      standup_tokens: 20000,
      current_mode: 'Reduced',
      auto_degrade_enabled: true,
      max_overhead_percent: 30,
      window_start: new Date().toISOString(),
    };

    const { container } = render(
      <CoordinationStatus
        metrics={metrics}
        onForceMode={mockForceMode}
      />
    );

    expect(screen.getByText(/40%/)).toBeInTheDocument();

    // Check that gauge bar has red color class
    const gaugeBar = container.querySelector('.bg-red-500');
    expect(gaugeBar).toBeInTheDocument();
  });

  it('displays token breakdown with formatted values', () => {
    const mockForceMode = vi.fn();

    const metrics: CoordinationMetrics = {
      coordination_tokens: 15000,
      production_tokens: 1500000,
      overhead_percent: 1,
      heartbeat_tokens: 8000,
      standup_tokens: 7000,
      current_mode: 'Full',
      auto_degrade_enabled: true,
      max_overhead_percent: 30,
      window_start: new Date().toISOString(),
    };

    render(
      <CoordinationStatus
        metrics={metrics}
        onForceMode={mockForceMode}
      />
    );

    // Check token breakdown (formatted with K/M)
    expect(screen.getByText(/heartbeat:/i)).toBeInTheDocument();
    expect(screen.getByText(/8\.0k/i)).toBeInTheDocument();

    expect(screen.getByText(/standup:/i)).toBeInTheDocument();
    expect(screen.getByText(/7\.0k/i)).toBeInTheDocument();

    expect(screen.getByText(/production:/i)).toBeInTheDocument();
    expect(screen.getByText(/1\.5m/i)).toBeInTheDocument();
  });

  it('shows auto-degrade enabled indicator', () => {
    const mockForceMode = vi.fn();

    const metrics: CoordinationMetrics = {
      coordination_tokens: 10000,
      production_tokens: 100000,
      overhead_percent: 10,
      heartbeat_tokens: 5000,
      standup_tokens: 5000,
      current_mode: 'Full',
      auto_degrade_enabled: true,
      max_overhead_percent: 30,
      window_start: new Date().toISOString(),
    };

    render(
      <CoordinationStatus
        metrics={metrics}
        onForceMode={mockForceMode}
      />
    );

    expect(screen.getByText(/auto-degrade:/i)).toBeInTheDocument();
    expect(screen.getByText('Enabled')).toBeInTheDocument();
  });

  it('shows auto-degrade manual indicator when disabled', () => {
    const mockForceMode = vi.fn();

    const metrics: CoordinationMetrics = {
      coordination_tokens: 10000,
      production_tokens: 100000,
      overhead_percent: 10,
      heartbeat_tokens: 5000,
      standup_tokens: 5000,
      current_mode: 'Full',
      auto_degrade_enabled: false,
      max_overhead_percent: 30,
      window_start: new Date().toISOString(),
    };

    render(
      <CoordinationStatus
        metrics={metrics}
        onForceMode={mockForceMode}
      />
    );

    expect(screen.getByText('Manual')).toBeInTheDocument();
  });

  it('allows mode selection from dropdown', () => {
    const mockForceMode = vi.fn();

    const metrics: CoordinationMetrics = {
      coordination_tokens: 10000,
      production_tokens: 100000,
      overhead_percent: 10,
      heartbeat_tokens: 5000,
      standup_tokens: 5000,
      current_mode: 'Full',
      auto_degrade_enabled: true,
      max_overhead_percent: 30,
      window_start: new Date().toISOString(),
    };

    render(
      <CoordinationStatus
        metrics={metrics}
        onForceMode={mockForceMode}
      />
    );

    // Click mode badge to open dropdown
    const modeButton = screen.getByText('Full');
    fireEvent.click(modeButton);

    // Dropdown should appear with all modes
    expect(screen.getByText('Standard')).toBeInTheDocument();
    expect(screen.getByText('Reduced')).toBeInTheDocument();
    expect(screen.getByText('HeartbeatOnly')).toBeInTheDocument();
    expect(screen.getByText('Disabled')).toBeInTheDocument();

    // Select a different mode
    const reducedOption = screen.getByText('Reduced');
    fireEvent.click(reducedOption);

    // Should call onForceMode
    expect(mockForceMode).toHaveBeenCalledWith('Reduced');
  });

  it('renders in compact mode', () => {
    const mockForceMode = vi.fn();

    const metrics: CoordinationMetrics = {
      coordination_tokens: 10000,
      production_tokens: 100000,
      overhead_percent: 15,
      heartbeat_tokens: 5000,
      standup_tokens: 5000,
      current_mode: 'Standard',
      auto_degrade_enabled: true,
      max_overhead_percent: 30,
      window_start: new Date().toISOString(),
    };

    render(
      <CoordinationStatus
        metrics={metrics}
        onForceMode={mockForceMode}
        compact={true}
      />
    );

    // Should show mode badge
    expect(screen.getByText('Standard')).toBeInTheDocument();

    // Should show overhead percentage
    expect(screen.getByText(/15% overhead/i)).toBeInTheDocument();

    // Should NOT show token breakdown (compact mode)
    expect(screen.queryByText(/heartbeat:/i)).not.toBeInTheDocument();
  });
});
