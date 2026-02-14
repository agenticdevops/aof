/**
 * Hook for fetching agents configuration from Phase 1 API.
 * Implements loading states, error handling, and version tracking.
 */

import { useEffect, useState, useCallback } from 'react';
import type { Agent } from '../types/events';

/**
 * Hook return type.
 */
interface UseAgentsConfigReturn {
  /** Configured agents */
  agents: Agent[];

  /** Configuration version */
  version: string;

  /** Loading state */
  loading: boolean;

  /** Error state */
  error: Error | null;

  /** Refetch function */
  refetch: () => void;
}

/**
 * Fetch agents configuration from API.
 *
 * @returns Agents config state and refetch function
 */
export function useAgentsConfig(): UseAgentsConfigReturn {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [version, setVersion] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchAgents = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch('/api/config/agents');

      if (!response.ok) {
        if (response.status === 404) {
          // Graceful fallback: empty array if endpoint doesn't exist yet
          console.warn('[useAgentsConfig] Endpoint not found, using empty config');
          setAgents([]);
          setVersion('');
          setLoading(false);
          return;
        }

        throw new Error(`Failed to fetch agents: ${response.statusText}`);
      }

      const data = await response.json();
      const configVersion = response.headers.get('X-Config-Version') || '';

      setAgents(data);
      setVersion(configVersion);
    } catch (err) {
      console.error('[useAgentsConfig] Fetch error:', err);
      setError(err instanceof Error ? err : new Error('Unknown error'));
      // Graceful fallback: empty array on error
      setAgents([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchAgents();
  }, [fetchAgents]);

  return {
    agents,
    version,
    loading,
    error,
    refetch: fetchAgents,
  };
}
