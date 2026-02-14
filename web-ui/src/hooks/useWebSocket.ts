/**
 * WebSocket hook with automatic reconnection and exponential backoff.
 * Connects to Phase 1 WebSocket endpoint and dispatches events to Redux.
 */

import { useEffect, useRef, useState } from 'react';
import { useDispatch } from 'react-redux';
import { addEvent, setConnected } from '../store/eventsSlice';
import {
  updateAgentHealth,
  addStandupResponse,
  updateStandupSummary,
} from '../store/coordinationSlice';
import type { CoordinationEvent } from '../types/events';
import type {
  HeartbeatResponsePayload,
  HeartbeatTimeoutPayload,
  StandupResponsePayload,
  StandupSummaryPayload,
  AgentHealthRecord,
} from '../types/coordination';

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

    /**
     * Handle coordination-specific WebSocket events.
     * Dispatches to coordinationSlice based on event type.
     */
    function handleCoordinationEvent(coordinationEvent: CoordinationEvent): void {
      const details = coordinationEvent.activity?.details;
      if (!details) return;

      // Check for coordination_activity field (Phase 7 coordination events)
      const coordActivity = details.coordination_activity;
      if (!coordActivity || typeof coordActivity !== 'object') return;

      const coordType = (coordActivity as Record<string, unknown>).type;

      switch (coordType) {
        case 'HeartbeatResponse': {
          // Parse HeartbeatResponse payload
          const payload = coordActivity as unknown as HeartbeatResponsePayload;
          const healthRecord: AgentHealthRecord = {
            agent_id: payload.agent_id,
            status: payload.status,
            last_heartbeat: payload.timestamp,
            consecutive_misses: 0,
            last_response_ms: payload.response_time_ms,
          };
          dispatch(updateAgentHealth(healthRecord));
          break;
        }

        case 'HeartbeatTimeout': {
          // Parse HeartbeatTimeout payload
          const payload = coordActivity as unknown as HeartbeatTimeoutPayload;
          // Mark each timed-out agent as Unresponsive
          payload.agent_ids.forEach((agentId) => {
            const healthRecord: AgentHealthRecord = {
              agent_id: agentId,
              status: 'Unresponsive',
              last_heartbeat: null,
              consecutive_misses: payload.consecutive_misses,
              last_response_ms: null,
            };
            dispatch(updateAgentHealth(healthRecord));
          });
          break;
        }

        case 'StandupResponse': {
          // Parse StandupResponse payload
          const payload = coordActivity as unknown as StandupResponsePayload;
          dispatch(addStandupResponse(payload.response));
          break;
        }

        case 'StandupSummary': {
          // Parse StandupSummary payload
          const payload = coordActivity as unknown as StandupSummaryPayload;
          dispatch(
            updateStandupSummary({
              request_id: payload.request_id,
              summary: payload.summary,
            })
          );
          break;
        }

        default:
          // Unknown coordination event type - ignore
          break;
      }
    }

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

            // Dispatch to events slice (existing behavior)
            dispatch(addEvent(coordinationEvent));
            setLastEvent(coordinationEvent);

            // Handle coordination-specific events (Phase 7-05)
            handleCoordinationEvent(coordinationEvent);
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
