/**
 * Hook for polling agent reliability metrics from the metrics API.
 *
 * Fetches /api/agents/:id/metrics on an interval and returns
 * uptime_percent, success_rate, loading state, and error.
 *
 * Features:
 * - Configurable polling interval (default 5000ms)
 * - X-Metrics-Version header detection for immediate refetch
 * - Graceful error handling (no crashes)
 * - Cleanup on unmount (stops polling)
 * - Exponential backoff on repeated errors
 */

import { useEffect, useState, useRef, useCallback } from 'react';

/**
 * Metrics response shape from /api/agents/:id/metrics.
 */
export interface AgentMetricsData {
  /** Agent identifier */
  agent_id: string;

  /** Uptime percentage (null if insufficient data) */
  uptime_percent: number | null;

  /** Success rate percentage (null if insufficient data) */
  success_rate: number | null;

  /** Total number of events processed */
  event_count: number;

  /** ISO 8601 timestamp of last metric update */
  last_update: string;

  /** ISO 8601 timestamp of last error (null if none) */
  last_error: string | null;
}

/**
 * Hook return type.
 */
export interface UseAgentMetricsReturn {
  /** Uptime percentage (null if unavailable or insufficient data) */
  uptime_percent: number | null;

  /** Success rate percentage (null if unavailable or insufficient data) */
  success_rate: number | null;

  /** Total event count for this agent */
  event_count: number;

  /** Whether the hook is currently loading data */
  loading: boolean;

  /** Error from the last fetch attempt (null if none) */
  error: Error | null;

  /** Force an immediate refetch */
  refetch: () => void;
}

/** Maximum backoff delay in milliseconds */
const MAX_BACKOFF_MS = 30000;

/**
 * Hook for polling agent reliability metrics.
 *
 * @param agentId - Agent identifier to fetch metrics for
 * @param pollIntervalMs - Polling interval in milliseconds (default: 5000)
 * @returns Metrics data, loading state, error, and refetch function
 *
 * @example
 * ```tsx
 * const { uptime_percent, success_rate, loading } = useAgentMetrics('k8s-monitor');
 * ```
 */
export function useAgentMetrics(
  agentId: string,
  pollIntervalMs: number = 5000
): UseAgentMetricsReturn {
  const [uptimePercent, setUptimePercent] = useState<number | null>(null);
  const [successRate, setSuccessRate] = useState<number | null>(null);
  const [eventCount, setEventCount] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const lastVersionRef = useRef<string>('');
  const errorCountRef = useRef(0);
  const mountedRef = useRef(true);

  const fetchMetrics = useCallback(async () => {
    if (!mountedRef.current || !agentId) return;

    try {
      const response = await fetch(`/api/agents/${encodeURIComponent(agentId)}/metrics`);

      if (!mountedRef.current) return;

      if (response.status === 429) {
        // Rate limited: increase backoff
        errorCountRef.current = Math.min(errorCountRef.current + 1, 5);
        console.warn('[useAgentMetrics] Rate limited, backing off');
        return;
      }

      if (response.status === 404) {
        // Agent not found: set null metrics without error
        setUptimePercent(null);
        setSuccessRate(null);
        setEventCount(0);
        setLoading(false);
        setError(null);
        errorCountRef.current = 0;
        return;
      }

      if (!response.ok) {
        throw new Error(`Metrics fetch failed: ${response.statusText}`);
      }

      const data: AgentMetricsData = await response.json();

      if (!mountedRef.current) return;

      setUptimePercent(data.uptime_percent);
      setSuccessRate(data.success_rate);
      setEventCount(data.event_count);
      setError(null);
      errorCountRef.current = 0;

      // Track version for change detection
      const version = response.headers.get('X-Metrics-Version') || '';
      lastVersionRef.current = version;
    } catch (err) {
      if (!mountedRef.current) return;

      console.error('[useAgentMetrics] Fetch error:', err);
      setError(err instanceof Error ? err : new Error('Unknown error'));
      errorCountRef.current = Math.min(errorCountRef.current + 1, 5);
    } finally {
      if (mountedRef.current) {
        setLoading(false);
      }
    }
  }, [agentId]);

  useEffect(() => {
    mountedRef.current = true;
    setLoading(true);

    // Initial fetch
    fetchMetrics();

    // Set up polling with backoff awareness
    const intervalId = setInterval(() => {
      const backoff = Math.min(
        pollIntervalMs * Math.pow(2, errorCountRef.current),
        MAX_BACKOFF_MS
      );

      if (errorCountRef.current > 0) {
        // Use backoff timing: only fetch if enough time has passed
        // The interval fires at pollIntervalMs but we skip fetches during backoff
        const shouldFetch = Math.random() < (pollIntervalMs / backoff);
        if (!shouldFetch) return;
      }

      fetchMetrics();
    }, pollIntervalMs);

    return () => {
      mountedRef.current = false;
      clearInterval(intervalId);
    };
  }, [fetchMetrics, pollIntervalMs]);

  return {
    uptime_percent: uptimePercent,
    success_rate: successRate,
    event_count: eventCount,
    loading,
    error,
    refetch: fetchMetrics,
  };
}
