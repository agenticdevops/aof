/**
 * Integration tests for KanbanBoard component.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { Provider } from 'react-redux';
import { configureStore } from '@reduxjs/toolkit';
import { KanbanBoard } from './KanbanBoard';
import eventsReducer from '../store/eventsSlice';
import configReducer from '../store/configSlice';
import tasksReducer from '../store/tasksSlice';
import type { Task } from '../types/tasks';

/**
 * Mock fetch globally.
 */
global.fetch = vi.fn();

/**
 * Create mock store for testing.
 */
function createMockStore(initialTasks: Task[] = []) {
  return configureStore({
    reducer: {
      events: eventsReducer,
      config: configReducer,
      tasks: tasksReducer,
    },
    preloadedState: {
      tasks: {
        tasks: {
          backlog: [],
          assigned: [],
          'in-progress': [],
          review: [],
          done: [],
        },
        optimisticTasks: {
          backlog: [],
          assigned: [],
          'in-progress': [],
          review: [],
          done: [],
        },
        pending: {},
        loading: false,
        error: null,
      },
    },
  });
}

describe('KanbanBoard', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render all 5 lanes', async () => {
    const store = createMockStore();

    // Mock fetch to return empty tasks
    (global.fetch as any).mockResolvedValueOnce({
      ok: true,
      json: async () => [],
    });

    render(
      <Provider store={store}>
        <KanbanBoard />
      </Provider>
    );

    // Wait for fetch to complete
    await waitFor(() => {
      expect(screen.getByText('Backlog')).toBeInTheDocument();
      expect(screen.getByText('Assigned')).toBeInTheDocument();
      expect(screen.getByText('In Progress')).toBeInTheDocument();
      expect(screen.getByText('Review')).toBeInTheDocument();
      expect(screen.getByText('Done')).toBeInTheDocument();
    });
  });

  it('should display empty state when no tasks', async () => {
    const store = createMockStore();

    (global.fetch as any).mockResolvedValueOnce({
      ok: true,
      json: async () => [],
    });

    render(
      <Provider store={store}>
        <KanbanBoard />
      </Provider>
    );

    await waitFor(() => {
      expect(screen.getAllByText(/No tasks in/i)).toHaveLength(5);
    });
  });

  it('should render tasks in correct lanes', async () => {
    const mockTasks: Task[] = [
      {
        id: 'task-1',
        title: 'Task in backlog',
        description: 'Description',
        lane: 'backlog',
        version: 1,
        status: 'pending',
        createdAt: '2024-02-14T10:00:00Z',
        updatedAt: '2024-02-14T10:00:00Z',
      },
      {
        id: 'task-2',
        title: 'Task in progress',
        description: 'Description',
        lane: 'in-progress',
        version: 1,
        status: 'active',
        createdAt: '2024-02-14T10:00:00Z',
        updatedAt: '2024-02-14T10:00:00Z',
      },
      {
        id: 'task-3',
        title: 'Task done',
        description: 'Description',
        lane: 'done',
        version: 1,
        status: 'completed',
        createdAt: '2024-02-14T10:00:00Z',
        updatedAt: '2024-02-14T10:00:00Z',
      },
    ];

    const store = createMockStore();

    (global.fetch as any).mockResolvedValueOnce({
      ok: true,
      json: async () => mockTasks,
    });

    render(
      <Provider store={store}>
        <KanbanBoard />
      </Provider>
    );

    await waitFor(() => {
      expect(screen.getByText('Task in backlog')).toBeInTheDocument();
      expect(screen.getByText('Task in progress')).toBeInTheDocument();
      expect(screen.getByText('Task done')).toBeInTheDocument();
    });
  });

  it('should handle fetch errors gracefully', async () => {
    const store = createMockStore();

    (global.fetch as any).mockRejectedValueOnce(new Error('Network error'));

    render(
      <Provider store={store}>
        <KanbanBoard />
      </Provider>
    );

    // Wait for fetch to fail and board to still render
    await waitFor(() => {
      expect(screen.getByText('Backlog')).toBeInTheDocument();
    });
  });

  it('should display keyboard shortcuts button', async () => {
    const store = createMockStore();

    (global.fetch as any).mockResolvedValueOnce({
      ok: true,
      json: async () => [],
    });

    render(
      <Provider store={store}>
        <KanbanBoard />
      </Provider>
    );

    await waitFor(() => {
      expect(screen.getByText('Keyboard Shortcuts')).toBeInTheDocument();
    });
  });
});
