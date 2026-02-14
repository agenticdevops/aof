/**
 * Hook for agent introduction toast notifications.
 * Subscribes to Redux introduction events and displays toasts
 * when agents introduce themselves for the first time.
 *
 * Uses localStorage to persist "don't show again" preference.
 */

import { useEffect, useCallback, useRef } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import type { RootState } from '../store';
import { consumeIntroduction } from '../store/configSlice';
import type { IntroductionMessage } from '../types/events';

/**
 * localStorage key for suppressing introduction toasts.
 */
const SUPPRESS_KEY = 'aof-suppress-introductions';

/**
 * Toast duration in milliseconds (8 seconds).
 */
const TOAST_DURATION = 8000;

/**
 * Maximum visible toasts at once.
 */
const MAX_VISIBLE_TOASTS = 3;

/**
 * Debounce interval per agent (1 second).
 */
const DEBOUNCE_MS = 1000;

/**
 * Active toast state for a single introduction.
 */
export interface IntroductionToast {
  /** Agent name */
  agentName: string;

  /** Introduction message text */
  message: string;

  /** Agent avatar emoji */
  avatar: string;

  /** Agent skills */
  skills: string[];

  /** Timestamp when toast was created */
  createdAt: number;
}

/**
 * Return type for the hook.
 */
export interface UseAgentIntroductionReturn {
  /** Currently active toasts (max 3) */
  activeToasts: IntroductionToast[];

  /** Dismiss a specific toast by agent name */
  dismissToast: (agentName: string) => void;

  /** Whether introductions are suppressed */
  isSuppressed: boolean;

  /** Toggle suppression preference */
  toggleSuppression: () => void;

  /** Navigate to agent card by ID */
  focusAgent: (agentName: string) => void;
}

/**
 * Hook for managing agent introduction toasts.
 *
 * Subscribes to configSlice.introductions and creates toast
 * notifications for new introductions.
 *
 * @example
 * ```tsx
 * function AgentGrid() {
 *   const { activeToasts, dismissToast } = useAgentIntroduction();
 *   // render toasts...
 * }
 * ```
 */
export function useAgentIntroduction(): UseAgentIntroductionReturn {
  const dispatch = useDispatch();
  const introductions = useSelector(
    (state: RootState) => state.config.introductions,
  );

  const activeToastsRef = useRef<IntroductionToast[]>([]);
  const lastIntroTimeRef = useRef<Record<string, number>>({});
  const suppressedRef = useRef(
    typeof window !== 'undefined' &&
      localStorage.getItem(SUPPRESS_KEY) === 'true',
  );

  /**
   * Process new introductions into toasts.
   */
  useEffect(() => {
    if (suppressedRef.current) return;
    if (introductions.length === 0) return;

    const now = Date.now();

    introductions.forEach((intro: IntroductionMessage) => {
      // Debounce: skip if introduced within last second
      const lastTime = lastIntroTimeRef.current[intro.agent_name];
      if (lastTime && now - lastTime < DEBOUNCE_MS) return;

      // Check if already in active toasts
      const alreadyActive = activeToastsRef.current.some(
        (t) => t.agentName === intro.agent_name,
      );
      if (alreadyActive) return;

      // Create toast
      const toast: IntroductionToast = {
        agentName: intro.agent_name,
        message: intro.intro_message,
        avatar: intro.avatar || '🤖',
        skills: intro.skills,
        createdAt: now,
      };

      // Enforce max visible limit by removing oldest
      if (activeToastsRef.current.length >= MAX_VISIBLE_TOASTS) {
        const oldest = activeToastsRef.current[0];
        activeToastsRef.current = activeToastsRef.current.slice(1);
        dispatch(consumeIntroduction(oldest.agentName));
      }

      activeToastsRef.current = [...activeToastsRef.current, toast];
      lastIntroTimeRef.current[intro.agent_name] = now;

      // Auto-dismiss after TOAST_DURATION
      setTimeout(() => {
        activeToastsRef.current = activeToastsRef.current.filter(
          (t) => t.agentName !== intro.agent_name,
        );
        dispatch(consumeIntroduction(intro.agent_name));
      }, TOAST_DURATION);
    });
  }, [introductions, dispatch]);

  /**
   * Dismiss a specific toast.
   */
  const dismissToast = useCallback(
    (agentName: string) => {
      activeToastsRef.current = activeToastsRef.current.filter(
        (t) => t.agentName !== agentName,
      );
      dispatch(consumeIntroduction(agentName));
    },
    [dispatch],
  );

  /**
   * Toggle suppression preference.
   */
  const toggleSuppression = useCallback(() => {
    const newValue = !suppressedRef.current;
    suppressedRef.current = newValue;
    if (typeof window !== 'undefined') {
      localStorage.setItem(SUPPRESS_KEY, String(newValue));
    }
  }, []);

  /**
   * Focus an agent card in the grid.
   */
  const focusAgent = useCallback((agentName: string) => {
    const card = document.querySelector(`[data-agent-id="${agentName}"]`);
    if (card instanceof HTMLElement) {
      card.scrollIntoView({ behavior: 'smooth', block: 'center' });
      card.focus();
    }
  }, []);

  return {
    activeToasts: activeToastsRef.current,
    dismissToast,
    isSuppressed: suppressedRef.current,
    toggleSuppression,
    focusAgent,
  };
}
