/**
 * Tests for dateUtils.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  formatRelativeTime,
  formatTime,
  formatDate,
  formatDateTime,
  formatFullDateTime,
} from '../dateUtils';

describe('dateUtils', () => {
  beforeEach(() => {
    // Mock current time to 2026-02-14T14:30:00Z
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-02-14T14:30:00Z'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('formatRelativeTime', () => {
    it('should format current time as "less than a minute ago"', () => {
      const now = new Date('2026-02-14T14:30:00Z');
      expect(formatRelativeTime(now)).toBe('less than a minute ago');
    });

    it('should format 1 hour ago correctly', () => {
      const oneHourAgo = new Date('2026-02-14T13:30:00Z');
      expect(formatRelativeTime(oneHourAgo)).toBe('about 1 hour ago');
    });

    it('should format 2 days ago correctly', () => {
      const twoDaysAgo = new Date('2026-02-12T14:30:00Z');
      expect(formatRelativeTime(twoDaysAgo)).toBe('2 days ago');
    });

    it('should handle ISO string timestamps', () => {
      const isoString = '2026-02-14T13:30:00Z';
      expect(formatRelativeTime(isoString)).toBe('about 1 hour ago');
    });

    it('should handle invalid timestamps gracefully', () => {
      expect(formatRelativeTime('invalid')).toBe('Unknown time');
    });
  });

  describe('formatTime', () => {
    it('should format time as HH:mm in local timezone', () => {
      const timestamp = new Date('2026-02-14T14:30:00Z');
      const result = formatTime(timestamp);
      // Result will be in local timezone, so just verify it's a valid time format
      expect(result).toMatch(/^\d{2}:\d{2}$/);
    });

    it('should handle ISO string timestamps', () => {
      const result = formatTime('2026-02-14T09:15:00Z');
      expect(result).toMatch(/^\d{2}:\d{2}$/);
    });

    it('should handle invalid timestamps gracefully', () => {
      expect(formatTime('invalid')).toBe('Invalid time');
    });
  });

  describe('formatDate', () => {
    it('should format date as MMM dd', () => {
      const timestamp = new Date('2026-02-14T14:30:00Z');
      expect(formatDate(timestamp)).toBe('Feb 14');
    });

    it('should handle ISO string timestamps', () => {
      expect(formatDate('2026-12-25T00:00:00Z')).toBe('Dec 25');
    });

    it('should handle invalid timestamps gracefully', () => {
      expect(formatDate('invalid')).toBe('Invalid date');
    });
  });

  describe('formatDateTime', () => {
    it('should format date and time as MMM dd, HH:mm in local timezone', () => {
      const timestamp = new Date('2026-02-14T14:30:00Z');
      const result = formatDateTime(timestamp);
      // Result will be in local timezone, verify format only
      expect(result).toMatch(/^[A-Z][a-z]{2} \d{2}, \d{2}:\d{2}$/);
    });

    it('should handle ISO string timestamps', () => {
      const result = formatDateTime('2026-12-25T18:45:00Z');
      expect(result).toMatch(/^[A-Z][a-z]{2} \d{2}, \d{2}:\d{2}$/);
    });

    it('should handle invalid timestamps gracefully', () => {
      expect(formatDateTime('invalid')).toBe('Invalid date time');
    });
  });

  describe('formatFullDateTime', () => {
    it('should format full date and time in local timezone', () => {
      const timestamp = new Date('2026-02-14T14:30:00Z');
      const result = formatFullDateTime(timestamp);
      // Result will be in local timezone, verify format only
      expect(result).toMatch(/^[A-Z][a-z]+ \d{1,2}, \d{4} at \d{1,2}:\d{2} [AP]M$/);
    });

    it('should handle ISO string timestamps', () => {
      const result = formatFullDateTime('2026-12-25T09:15:00Z');
      expect(result).toMatch(/^[A-Z][a-z]+ \d{1,2}, \d{4} at \d{1,2}:\d{2} [AP]M$/);
    });

    it('should handle invalid timestamps gracefully', () => {
      expect(formatFullDateTime('invalid')).toBe('Invalid date time');
    });
  });
});
