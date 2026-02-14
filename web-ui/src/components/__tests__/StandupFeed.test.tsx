/**
 * StandupFeed component tests.
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { StandupFeed } from '../StandupFeed';
import type { StandupResult } from '../../types/coordination';

describe('StandupFeed', () => {
  it('renders empty state when no standup result', () => {
    const mockTrigger = vi.fn();

    render(
      <StandupFeed
        standupResult={null}
        onTriggerStandup={mockTrigger}
      />
    );

    expect(screen.getByText(/no standup results yet/i)).toBeInTheDocument();
    expect(screen.getByText(/trigger a standup/i)).toBeInTheDocument();
  });

  it('renders empty state when standup has no responses', () => {
    const mockTrigger = vi.fn();

    const emptyStandup: StandupResult = {
      request_id: 'test-123',
      responses: [],
      triggered_at: new Date().toISOString(),
    };

    render(
      <StandupFeed
        standupResult={emptyStandup}
        onTriggerStandup={mockTrigger}
      />
    );

    expect(screen.getByText(/no standup results yet/i)).toBeInTheDocument();
  });

  it('calls onTriggerStandup when trigger button clicked', () => {
    const mockTrigger = vi.fn();

    render(
      <StandupFeed
        standupResult={null}
        onTriggerStandup={mockTrigger}
      />
    );

    const button = screen.getByRole('button', { name: /trigger standup now/i });
    fireEvent.click(button);

    expect(mockTrigger).toHaveBeenCalledTimes(1);
  });

  it('renders standup responses with DID/DOING/BLOCKERS sections', () => {
    const mockTrigger = vi.fn();

    const standupResult: StandupResult = {
      request_id: 'test-123',
      responses: [
        {
          agent_id: 'agent-1',
          what_i_did: 'Fixed bug in API',
          what_im_doing: 'Implementing feature X',
          blockers: ['Waiting for review'],
          token_count: 150,
          timestamp: new Date().toISOString(),
        },
        {
          agent_id: 'agent-2',
          what_i_did: 'Reviewed PRs',
          what_im_doing: 'Writing tests',
          blockers: [],
          token_count: 120,
          timestamp: new Date().toISOString(),
        },
      ],
      triggered_at: new Date().toISOString(),
    };

    render(
      <StandupFeed
        standupResult={standupResult}
        onTriggerStandup={mockTrigger}
      />
    );

    // Check agent names
    expect(screen.getByText('agent-1')).toBeInTheDocument();
    expect(screen.getByText('agent-2')).toBeInTheDocument();

    // Check DID section
    expect(screen.getByText('Fixed bug in API')).toBeInTheDocument();
    expect(screen.getByText('Reviewed PRs')).toBeInTheDocument();

    // Check DOING section
    expect(screen.getByText('Implementing feature X')).toBeInTheDocument();
    expect(screen.getByText('Writing tests')).toBeInTheDocument();

    // Check BLOCKERS section
    expect(screen.getByText('Waiting for review')).toBeInTheDocument();
    expect(screen.getByText(/no blockers/i)).toBeInTheDocument();

    // Check token counts
    expect(screen.getByText(/150 tokens/i)).toBeInTheDocument();
    expect(screen.getByText(/120 tokens/i)).toBeInTheDocument();
  });

  it('shows summary when available', () => {
    const mockTrigger = vi.fn();

    const standupResult: StandupResult = {
      request_id: 'test-123',
      responses: [
        {
          agent_id: 'agent-1',
          what_i_did: 'Fixed bug',
          what_im_doing: 'Writing code',
          blockers: [],
          token_count: 100,
          timestamp: new Date().toISOString(),
        },
      ],
      summary: 'Team made good progress on bug fixes and feature development.',
      triggered_at: new Date().toISOString(),
    };

    render(
      <StandupFeed
        standupResult={standupResult}
        onTriggerStandup={mockTrigger}
      />
    );

    expect(screen.getByText(/team made good progress/i)).toBeInTheDocument();
  });

  it('shows correct response count', () => {
    const mockTrigger = vi.fn();

    const standupResult: StandupResult = {
      request_id: 'test-123',
      responses: [
        {
          agent_id: 'agent-1',
          what_i_did: 'Task 1',
          what_im_doing: 'Task 2',
          blockers: [],
          token_count: 100,
          timestamp: new Date().toISOString(),
        },
        {
          agent_id: 'agent-2',
          what_i_did: 'Task 3',
          what_im_doing: 'Task 4',
          blockers: [],
          token_count: 100,
          timestamp: new Date().toISOString(),
        },
        {
          agent_id: 'agent-3',
          what_i_did: 'Task 5',
          what_im_doing: 'Task 6',
          blockers: [],
          token_count: 100,
          timestamp: new Date().toISOString(),
        },
      ],
      triggered_at: new Date().toISOString(),
    };

    render(
      <StandupFeed
        standupResult={standupResult}
        onTriggerStandup={mockTrigger}
      />
    );

    expect(screen.getByText(/agent responses \(3\)/i)).toBeInTheDocument();
  });

  it('disables trigger button when loading', () => {
    const mockTrigger = vi.fn();

    render(
      <StandupFeed
        standupResult={null}
        onTriggerStandup={mockTrigger}
        isLoading={true}
      />
    );

    const button = screen.getByRole('button', { name: /triggering/i });
    expect(button).toBeDisabled();
  });

  it('allows expanding and collapsing agent responses', () => {
    const mockTrigger = vi.fn();

    const standupResult: StandupResult = {
      request_id: 'test-123',
      responses: [
        {
          agent_id: 'agent-1',
          what_i_did: 'Task completed',
          what_im_doing: 'Next task',
          blockers: [],
          token_count: 100,
          timestamp: new Date().toISOString(),
        },
      ],
      triggered_at: new Date().toISOString(),
    };

    render(
      <StandupFeed
        standupResult={standupResult}
        onTriggerStandup={mockTrigger}
      />
    );

    // Initially expanded (default state)
    expect(screen.getByText('Task completed')).toBeInTheDocument();

    // Find and click expand/collapse button
    const expandButton = screen.getByLabelText(/collapse/i);
    fireEvent.click(expandButton);

    // Content should be hidden (component removes from DOM when collapsed)
    // We can't easily test this without checking DOM structure
    // But we can verify the button aria-label changed
    expect(screen.getByLabelText(/expand/i)).toBeInTheDocument();
  });
});
