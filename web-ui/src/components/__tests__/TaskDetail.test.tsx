/**
 * Integration tests for TaskDetail modal component.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Provider } from 'react-redux';
import { configureStore } from '@reduxjs/toolkit';
import { TaskDetail } from '../TaskDetail';
import tasksReducer, { setTasks } from '../../store/tasksSlice';
import activitiesReducer from '../../store/activitiesSlice';
import type { Task } from '../../types/tasks';

describe('TaskDetail', () => {
  let store: ReturnType<typeof configureStore>;
  let mockTask: Task;

  beforeEach(() => {
    // Reset store before each test
    store = configureStore({
      reducer: {
        tasks: tasksReducer,
        activities: activitiesReducer,
      },
    });

    mockTask = {
      id: 'task_1',
      title: 'Test Task',
      description: 'This is a test task description',
      lane: 'in-progress',
      priority: 'high',
      assignee: 'Test Agent',
      tags: ['backend', 'api'],
      version: 1,
    };

    // Add task to store
    store.dispatch(setTasks([mockTask]));
  });

  it('should not render when taskId is null', () => {
    render(
      <Provider store={store}>
        <TaskDetail taskId={null} onClose={() => {}} />
      </Provider>
    );

    // Modal should not be visible
    expect(screen.queryByText('Test Task')).not.toBeInTheDocument();
  });

  it('should render modal when taskId is provided', () => {
    render(
      <Provider store={store}>
        <TaskDetail taskId="task_1" onClose={() => {}} />
      </Provider>
    );

    expect(screen.getByText('Test Task')).toBeInTheDocument();
  });

  it('should display all task details in Overview tab', () => {
    render(
      <Provider store={store}>
        <TaskDetail taskId="task_1" onClose={() => {}} />
      </Provider>
    );

    expect(screen.getByText('This is a test task description')).toBeInTheDocument();
    expect(screen.getByText('Test Agent')).toBeInTheDocument();
    expect(screen.getByText('high')).toBeInTheDocument();
    expect(screen.getByText('backend')).toBeInTheDocument();
    expect(screen.getByText('api')).toBeInTheDocument();
  });

  it('should have all three tabs (Overview, Comments, History)', () => {
    render(
      <Provider store={store}>
        <TaskDetail taskId="task_1" onClose={() => {}} />
      </Provider>
    );

    expect(screen.getByRole('tab', { name: /overview/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /comments/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /history/i })).toBeInTheDocument();
  });

  it('should switch tabs when clicked', async () => {
    const user = userEvent.setup();

    render(
      <Provider store={store}>
        <TaskDetail taskId="task_1" onClose={() => {}} />
      </Provider>
    );

    // Click Comments tab
    await user.click(screen.getByRole('tab', { name: /comments/i }));

    // Comments tab should be active
    const commentsTab = screen.getByRole('tab', { name: /comments/i });
    expect(commentsTab).toHaveAttribute('aria-selected', 'true');
  });

  it('should be keyboard accessible', () => {
    render(
      <Provider store={store}>
        <TaskDetail taskId="task_1" onClose={() => {}} />
      </Provider>
    );

    // Modal should have role="dialog"
    const modal = screen.getByRole('dialog');
    expect(modal).toBeInTheDocument();
    expect(modal).toHaveAttribute('aria-modal', 'true');

    // Close button should have aria-label
    const closeButton = screen.getByLabelText('Close modal');
    expect(closeButton).toBeInTheDocument();
  });
});
