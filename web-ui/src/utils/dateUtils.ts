/**
 * Date and time formatting utilities using date-fns.
 * All formats display in user's local timezone (UTC timestamps from server converted).
 */

import { formatDistanceToNow, format, parseISO } from 'date-fns';

/**
 * Format timestamp as relative time (e.g., "2 minutes ago").
 * Handles both ISO strings and Date objects.
 */
export function formatRelativeTime(timestamp: string | Date): string {
  try {
    const date = typeof timestamp === 'string' ? parseISO(timestamp) : timestamp;
    return formatDistanceToNow(date, { addSuffix: true });
  } catch (error) {
    console.error('Error formatting relative time:', error);
    return 'Unknown time';
  }
}

/**
 * Format timestamp as time only (e.g., "14:30").
 */
export function formatTime(timestamp: string | Date): string {
  try {
    const date = typeof timestamp === 'string' ? parseISO(timestamp) : timestamp;
    return format(date, 'HH:mm');
  } catch (error) {
    console.error('Error formatting time:', error);
    return 'Invalid time';
  }
}

/**
 * Format timestamp as date only (e.g., "Feb 14").
 */
export function formatDate(timestamp: string | Date): string {
  try {
    const date = typeof timestamp === 'string' ? parseISO(timestamp) : timestamp;
    return format(date, 'MMM dd');
  } catch (error) {
    console.error('Error formatting date:', error);
    return 'Invalid date';
  }
}

/**
 * Format timestamp as date and time (e.g., "Feb 14, 14:30").
 */
export function formatDateTime(timestamp: string | Date): string {
  try {
    const date = typeof timestamp === 'string' ? parseISO(timestamp) : timestamp;
    return format(date, 'MMM dd, HH:mm');
  } catch (error) {
    console.error('Error formatting date time:', error);
    return 'Invalid date time';
  }
}

/**
 * Format timestamp as full date and time (e.g., "February 14, 2026 at 2:30 PM").
 */
export function formatFullDateTime(timestamp: string | Date): string {
  try {
    const date = typeof timestamp === 'string' ? parseISO(timestamp) : timestamp;
    return format(date, 'MMMM dd, yyyy \'at\' h:mm a');
  } catch (error) {
    console.error('Error formatting full date time:', error);
    return 'Invalid date time';
  }
}
