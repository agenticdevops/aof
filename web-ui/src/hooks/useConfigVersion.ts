/**
 * Hook for polling configuration version changes.
 * Triggers refetch when version changes (placeholder implementation for dev).
 */

import { useEffect, useRef } from 'react';

/**
 * Poll configuration version endpoint.
 *
 * @param onVersionChange - Callback when version changes
 * @param intervalMs - Polling interval in milliseconds (default: 10000)
 */
export function useConfigVersion(
  onVersionChange: () => void,
  intervalMs: number = 10000
): void {
  const lastVersionRef = useRef<string>('');

  useEffect(() => {
    async function checkVersion() {
      try {
        const response = await fetch('/api/config/version');

        if (!response.ok) {
          // Graceful handling: don't spam errors in console
          return;
        }

        const data = await response.json();
        const currentVersion = data.version || '';

        if (lastVersionRef.current && currentVersion !== lastVersionRef.current) {
          console.log('[useConfigVersion] Version changed, triggering refetch');
          onVersionChange();
        }

        lastVersionRef.current = currentVersion;
      } catch (err) {
        // Silently ignore errors in dev mode
        // In production, consider logging to monitoring service
      }
    }

    // Initial check
    checkVersion();

    // Set up polling
    const intervalId = setInterval(checkVersion, intervalMs);

    return () => {
      clearInterval(intervalId);
    };
  }, [onVersionChange, intervalMs]);
}
