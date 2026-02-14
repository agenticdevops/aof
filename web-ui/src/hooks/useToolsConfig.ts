/**
 * Hook for fetching tools configuration from Phase 1 API.
 * Implements loading states, error handling, and version tracking.
 */

import { useEffect, useState, useCallback } from 'react';
import type { Tool } from '../types/events';

/**
 * Hook return type.
 */
interface UseToolsConfigReturn {
  /** Available tools */
  tools: Tool[];

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
 * Fetch tools configuration from API.
 *
 * @returns Tools config state and refetch function
 */
export function useToolsConfig(): UseToolsConfigReturn {
  const [tools, setTools] = useState<Tool[]>([]);
  const [version, setVersion] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchTools = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch('/api/config/tools');

      if (!response.ok) {
        if (response.status === 404) {
          // Graceful fallback: empty array if endpoint doesn't exist yet
          console.warn('[useToolsConfig] Endpoint not found, using empty config');
          setTools([]);
          setVersion('');
          setLoading(false);
          return;
        }

        throw new Error(`Failed to fetch tools: ${response.statusText}`);
      }

      const data = await response.json();
      const configVersion = response.headers.get('X-Config-Version') || '';

      setTools(data);
      setVersion(configVersion);
    } catch (err) {
      console.error('[useToolsConfig] Fetch error:', err);
      setError(err instanceof Error ? err : new Error('Unknown error'));
      // Graceful fallback: empty array on error
      setTools([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchTools();
  }, [fetchTools]);

  return {
    tools,
    version,
    loading,
    error,
    refetch: fetchTools,
  };
}
