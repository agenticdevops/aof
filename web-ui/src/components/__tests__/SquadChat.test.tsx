/**
 * Integration tests for SquadChat component.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Provider } from 'react-redux';
import { configureStore } from '@reduxjs/toolkit';
import { SquadChat } from '../SquadChat';
import chatReducer, { addMessage } from '../../store/chatSlice';
import eventsReducer from '../../store/eventsSlice';

// Mock fetch
global.fetch = vi.fn();

describe('SquadChat', () => {
  let store: ReturnType<typeof configureStore>;

  beforeEach(() => {
    // Reset store before each test
    store = configureStore({
      reducer: {
        chat: chatReducer,
        events: eventsReducer,
      },
    });

    // Reset fetch mock
    vi.clearAllMocks();
  });

  it('should render chat panel with header', () => {
    render(
      <Provider store={store}>
        <SquadChat />
      </Provider>
    );

    expect(screen.getByText('Squad Chat')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Type a message...')).toBeInTheDocument();
  });

  it('should display empty state when no messages', async () => {
    // Mock fetch to return empty array
    (global.fetch as any).mockResolvedValueOnce({
      ok: true,
      json: async () => [],
    });

    render(
      <Provider store={store}>
        <SquadChat />
      </Provider>
    );

    await waitFor(() => {
      expect(screen.getByText(/No messages yet/i)).toBeInTheDocument();
    });
  });

  it('should display messages when they exist', async () => {
    // Dispatch a message to the store
    store.dispatch(
      addMessage({
        id: 'msg_1',
        senderId: 'agent_1',
        senderName: 'Test Agent',
        content: 'Hello, world!',
        timestamp: new Date().toISOString(),
      })
    );

    // Mock fetch to return empty (already have message in state)
    (global.fetch as any).mockResolvedValueOnce({
      ok: true,
      json: async () => [],
    });

    render(
      <Provider store={store}>
        <SquadChat />
      </Provider>
    );

    expect(screen.getByText('Test Agent')).toBeInTheDocument();
    expect(screen.getByText('Hello, world!')).toBeInTheDocument();
  });

  it('should handle message deduplication', async () => {
    const message = {
      id: 'msg_1',
      senderId: 'agent_1',
      senderName: 'Test Agent',
      content: 'Duplicate test',
      timestamp: new Date().toISOString(),
    };

    // Dispatch same message twice
    store.dispatch(addMessage(message));
    store.dispatch(addMessage(message));

    // Mock fetch
    (global.fetch as any).mockResolvedValueOnce({
      ok: true,
      json: async () => [],
    });

    render(
      <Provider store={store}>
        <SquadChat />
      </Provider>
    );

    // Message should appear only once
    const messages = screen.getAllByText('Duplicate test');
    expect(messages).toHaveLength(1);
  });

  it('should disable send button when not connected', async () => {
    // Mock fetch
    (global.fetch as any).mockResolvedValueOnce({
      ok: true,
      json: async () => [],
    });

    render(
      <Provider store={store}>
        <SquadChat />
      </Provider>
    );

    const sendButton = screen.getByRole('button', { name: /send/i });
    expect(sendButton).toBeDisabled();
  });

  it('should be keyboard accessible', async () => {
    // Mock fetch
    (global.fetch as any).mockResolvedValueOnce({
      ok: true,
      json: async () => [],
    });

    render(
      <Provider store={store}>
        <SquadChat />
      </Provider>
    );

    const input = screen.getByPlaceholderText('Type a message...');

    // Tab navigation should reach input
    expect(input).toBeInTheDocument();

    // Input should have aria-label
    expect(input).toHaveAttribute('aria-label', 'Message input');

    // Send button should have aria-label
    const sendButton = screen.getByRole('button', { name: /send/i });
    expect(sendButton).toHaveAttribute('aria-label', 'Send message');
  });
});
