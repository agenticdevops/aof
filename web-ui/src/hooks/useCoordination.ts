/**
 * useCoordination hook - fetches and manages coordination state.
 * Provides actions for triggering standup and forcing coordination mode.
 */

import { useEffect, useCallback, useRef } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import {
  setHealth,
  setLatestStandup,
  setMetrics,
  setLoading,
  setError,
  selectCoordinationHealth,
  selectLatestStandup,
  selectCoordinationMetrics,
  selectCoordinationLoading,
  selectCoordinationError,
  selectCoordinationEnabled,
} from '../store/coordinationSlice';
import type {
  HeartbeatHealthResponse,
  StandupResult,
  CoordinationMetrics,
  CoordinationMode,
  AgentHealthRecord,
} from '../types/coordination';

/**
 * API base URL (from environment or default).
 */
const API_BASE =
  import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api';

/**
 * Hook return type.
 */
interface UseCoordinationReturn {
  /** Current agent health records */
  health: AgentHealthRecord[];

  /** Latest standup result */
  latestStandup: StandupResult | null;

  /** Coordination metrics */
  metrics: CoordinationMetrics | null;

  /** Loading state */
  isLoading: boolean;

  /** Error message */
  error: string | null;

  /** Whether coordination is enabled */
  coordinationEnabled: boolean;

  /** Trigger standup now */
  triggerStandup: () => Promise<void>;

  /** Force coordination mode */
  forceMode: (mode: CoordinationMode) => Promise<void>;

  /** Refresh health data */
  refreshHealth: () => Promise<void>;

  /** Refresh metrics data */
  refreshMetrics: () => Promise<void>;
}

/**
 * Custom hook for coordination state and actions.
 * Fetches initial data on mount and sets up polling for metrics.
 *
 * @param metricsPollingInterval - Metrics polling interval in milliseconds (default: 30000 = 30s)
 * @returns Coordination state and action functions
 *
 * @example
 * ```tsx
 * function CoordinationPage() {
 *   const {
 *     health,
 *     latestStandup,
 *     metrics,
 *     triggerStandup,
 *     forceMode,
 *   } = useCoordination();
 *
 *   return (
 *     <div>
 *       <button onClick={triggerStandup}>Trigger Standup</button>
 *       <button onClick={() => forceMode('Reduced')}>Reduce Mode</button>
 *     </div>
 *   );
 * }
 * ```
 */
export function useCoordination(
  metricsPollingInterval = 30000
): UseCoordinationReturn {
  const dispatch = useDispatch();

  // Select state from Redux
  const health = useSelector(selectCoordinationHealth);
  const latestStandup = useSelector(selectLatestStandup);
  const metrics = useSelector(selectCoordinationMetrics);
  const isLoading = useSelector(selectCoordinationLoading);
  const error = useSelector(selectCoordinationError);
  const coordinationEnabled = useSelector(selectCoordinationEnabled);

  const metricsIntervalRef = useRef<number | null>(null);

  /**
   * Fetch agent health from /api/coordination/health.
   */
  const fetchHealth = useCallback(async () => {
    try {
      const response = await fetch(`${API_BASE}/coordination/health`);

      if (!response.ok) {
        throw new Error(`Health fetch failed: ${response.statusText}`);
      }

      const data: HeartbeatHealthResponse = await response.json();
      dispatch(setHealth(data.agents));
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Unknown error';
      console.error('[useCoordination] Health fetch error:', message);
      dispatch(setError(message));
    }
  }, [dispatch]);

  /**
   * Fetch latest standup from /api/coordination/standup/latest.
   */
  const fetchLatestStandup = useCallback(async () => {
    try {
      const response = await fetch(`${API_BASE}/coordination/standup/latest`);

      if (response.status === 404) {
        // No standup results yet - not an error
        dispatch(setLatestStandup({
          request_id: '',
          responses: [],
          triggered_at: new Date().toISOString(),
        }));
        return;
      }

      if (!response.ok) {
        throw new Error(`Standup fetch failed: ${response.statusText}`);
      }

      const data: StandupResult = await response.json();
      dispatch(setLatestStandup(data));
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Unknown error';
      console.error('[useCoordination] Standup fetch error:', message);
      dispatch(setError(message));
    }
  }, [dispatch]);

  /**
   * Fetch coordination metrics from /api/coordination/metrics.
   */
  const fetchMetrics = useCallback(async () => {
    try {
      const response = await fetch(`${API_BASE}/coordination/metrics`);

      if (!response.ok) {
        throw new Error(`Metrics fetch failed: ${response.statusText}`);
      }

      const data: CoordinationMetrics = await response.json();
      dispatch(setMetrics(data));
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Unknown error';
      console.error('[useCoordination] Metrics fetch error:', message);
      dispatch(setError(message));
    }
  }, [dispatch]);

  /**
   * Trigger standup now (POST /api/coordination/standup/trigger).
   */
  const triggerStandup = useCallback(async () => {
    try {
      dispatch(setLoading(true));

      const response = await fetch(`${API_BASE}/coordination/standup/trigger`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
      });

      if (!response.ok) {
        throw new Error(`Standup trigger failed: ${response.statusText}`);
      }

      // Standup result will arrive via WebSocket events
      // Refresh latest standup after a short delay
      setTimeout(() => {
        fetchLatestStandup();
      }, 2000);

      dispatch(setLoading(false));
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Unknown error';
      console.error('[useCoordination] Standup trigger error:', message);
      dispatch(setError(message));
    }
  }, [dispatch, fetchLatestStandup]);

  /**
   * Force coordination mode (POST /api/coordination/mode).
   */
  const forceMode = useCallback(
    async (mode: CoordinationMode) => {
      try {
        dispatch(setLoading(true));

        const response = await fetch(`${API_BASE}/coordination/mode`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ mode }),
        });

        if (!response.ok) {
          throw new Error(`Mode change failed: ${response.statusText}`);
        }

        // Refresh metrics to get updated mode
        await fetchMetrics();

        dispatch(setLoading(false));
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Unknown error';
        console.error('[useCoordination] Mode change error:', message);
        dispatch(setError(message));
      }
    },
    [dispatch, fetchMetrics]
  );

  /**
   * Refresh health data manually.
   */
  const refreshHealth = useCallback(async () => {
    dispatch(setLoading(true));
    await fetchHealth();
    dispatch(setLoading(false));
  }, [dispatch, fetchHealth]);

  /**
   * Refresh metrics data manually.
   */
  const refreshMetrics = useCallback(async () => {
    dispatch(setLoading(true));
    await fetchMetrics();
    dispatch(setLoading(false));
  }, [dispatch, fetchMetrics]);

  /**
   * Initial data fetch on mount.
   */
  useEffect(() => {
    const loadInitialData = async () => {
      dispatch(setLoading(true));

      await Promise.all([
        fetchHealth(),
        fetchLatestStandup(),
        fetchMetrics(),
      ]);

      dispatch(setLoading(false));
    };

    loadInitialData();
  }, [dispatch, fetchHealth, fetchLatestStandup, fetchMetrics]);

  /**
   * Set up metrics polling (every metricsPollingInterval ms).
   */
  useEffect(() => {
    if (metricsPollingInterval > 0) {
      metricsIntervalRef.current = window.setInterval(() => {
        fetchMetrics();
      }, metricsPollingInterval);
    }

    return () => {
      if (metricsIntervalRef.current !== null) {
        clearInterval(metricsIntervalRef.current);
      }
    };
  }, [metricsPollingInterval, fetchMetrics]);

  return {
    health,
    latestStandup,
    metrics,
    isLoading,
    error,
    coordinationEnabled,
    triggerStandup,
    forceMode,
    refreshHealth,
    refreshMetrics,
  };
}
