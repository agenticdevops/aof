/**
 * Integration tests for ActivityFeed component.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Provider } from 'react-redux';
import { configureStore } from '@reduxjs/toolkit';
import { ActivityFeed } from '../ActivityFeed';
import activitiesReducer, { addActivity } from '../../store/activitiesSlice';
import eventsReducer from '../../store/eventsSlice';

describe('ActivityFeed', () => {
  let store: ReturnType<typeof configureStore>;

  beforeEach(() => {
    // Reset store before each test
    store = configureStore({
      reducer: {
        activities: activitiesReducer,
        events: eventsReducer,
      },
    });
  });

  it('should render activity feed with header', () => {
    render(
      <Provider store={store}>
        <ActivityFeed />
      </Provider>
    );

    expect(screen.getByText('Activity Feed')).toBeInTheDocument();
  });

  it('should display empty state when no activities', () => {
    render(
      <Provider store={store}>
        <ActivityFeed />
      </Provider>
    );

    expect(screen.getByText(/No activity yet/i)).toBeInTheDocument();
  });

  it('should render activity item when added', () => {
    // Dispatch activity to store
    store.dispatch(
      addActivity({
        eventId: 'evt_1',
        agentId: 'agent_1',
        agentName: 'Test Agent',
        activityType: 'agent_started',
        description: 'Test Agent started execution',
        details: {},
        timestamp: new Date().toISOString(),
        icon: '▶️',
        color: 'blue',
      })
    );

    render(
      <Provider store={store}>
        <ActivityFeed />
      </Provider>
    );

    expect(screen.getByText('Test Agent started execution')).toBeInTheDocument();
    expect(screen.getByText(/Agent: Test Agent/i)).toBeInTheDocument();
  });

  it('should display multiple activities', () => {
    const now = Date.now();

    // Add activities with different timestamps
    store.dispatch(
      addActivity({
        eventId: 'evt_1',
        agentId: 'agent_1',
        agentName: 'Agent 1',
        activityType: 'agent_started',
        description: 'First activity event',
        details: {},
        timestamp: new Date(now - 2000).toISOString(),
        icon: '▶️',
        color: 'blue',
      })
    );

    store.dispatch(
      addActivity({
        eventId: 'evt_2',
        agentId: 'agent_2',
        agentName: 'Agent 2',
        activityType: 'agent_completed',
        description: 'Second activity event',
        details: {},
        timestamp: new Date(now - 1000).toISOString(),
        icon: '✅',
        color: 'green',
      })
    );

    render(
      <Provider store={store}>
        <ActivityFeed />
      </Provider>
    );

    // Verify both activities are present
    expect(screen.getByText('First activity event')).toBeInTheDocument();
    expect(screen.getByText('Second activity event')).toBeInTheDocument();
  });

  it('should be keyboard accessible', () => {
    store.dispatch(
      addActivity({
        eventId: 'evt_1',
        agentId: 'agent_1',
        agentName: 'Test Agent',
        activityType: 'tool_called',
        description: 'Test activity',
        details: { tool_name: 'test_tool' },
        timestamp: new Date().toISOString(),
        icon: '🔧',
        color: 'blue',
      })
    );

    render(
      <Provider store={store}>
        <ActivityFeed />
      </Provider>
    );

    // Activity item should be a button (expandable)
    const activityButton = screen.getByRole('button', { name: /Activity: Test activity/i });
    expect(activityButton).toBeInTheDocument();
    expect(activityButton).toHaveAttribute('aria-expanded', 'false');
  });
});
