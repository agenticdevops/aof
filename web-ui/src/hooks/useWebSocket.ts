/**
 * WebSocket hook with automatic reconnection and exponential backoff.
 * Connects to Phase 1 WebSocket endpoint and dispatches events to Redux.
 */

import { useEffect, useRef, useState } from 'react';
import { useDispatch } from 'react-redux';
import { addEvent, setConnected } from '../store/eventsSlice';
import type { CoordinationEvent } from '../types/events';

/**
 * Hook return type.
 */
interface UseWebSocketReturn {
  /** Connection status */
  connected: boolean;

  /** Last received event */
  lastEvent: CoordinationEvent | null;

  /** Number of reconnection attempts */
  reconnectAttempts: number;
}

/**
 * WebSocket hook with automatic reconnection.
 *
 * @param url - WebSocket URL (e.g., ws://localhost:8080/ws)
 * @returns Connection state and last event
 */
export function useWebSocket(url: string): UseWebSocketReturn {
  const dispatch = useDispatch();
  const [connected, setConnectedState] = useState(false);
  const [lastEvent, setLastEvent] = useState<CoordinationEvent | null>(null);
  const [reconnectAttempts, setReconnectAttempts] = useState(0);

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<number | null>(null);
  const retryCountRef = useRef(0);

  useEffect(() => {
    let shouldReconnect = true;

    function connect() {
      try {
        const ws = new WebSocket(url);
        wsRef.current = ws;

        ws.onopen = () => {
          console.log('[WebSocket] Connected to', url);
          setConnectedState(true);
          dispatch(setConnected(true));
          retryCountRef.current = 0;
          setReconnectAttempts(0);
        };

        ws.onmessage = (event) => {
          try {
            const coordinationEvent: CoordinationEvent = JSON.parse(event.data);
            console.log('[WebSocket] Event received:', coordinationEvent);
            dispatch(addEvent(coordinationEvent));
            setLastEvent(coordinationEvent);
          } catch (error) {
            console.error('[WebSocket] Failed to parse event:', error);
          }
        };

        ws.onerror = (error) => {
          console.error('[WebSocket] Error:', error);
        };

        ws.onclose = () => {
          console.log('[WebSocket] Connection closed');
          setConnectedState(false);
          dispatch(setConnected(false));
          wsRef.current = null;

          // Exponential backoff: 1s, 2s, 4s, 8s, 16s, 30s (cap)
          if (shouldReconnect) {
            const delay = Math.min(1000 * Math.pow(2, retryCountRef.current), 30000);
            retryCountRef.current += 1;
            setReconnectAttempts(retryCountRef.current);

            console.log(`[WebSocket] Reconnecting in ${delay}ms (attempt ${retryCountRef.current})`);

            reconnectTimeoutRef.current = window.setTimeout(() => {
              connect();
            }, delay);
          }
        };
      } catch (error) {
        console.error('[WebSocket] Connection failed:', error);
      }
    }

    connect();

    // Cleanup on unmount
    return () => {
      shouldReconnect = false;
      if (reconnectTimeoutRef.current !== null) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [url, dispatch]);

  return { connected, lastEvent, reconnectAttempts };
}
