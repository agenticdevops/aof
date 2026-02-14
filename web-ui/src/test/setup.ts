/**
 * Vitest setup file.
 * Configures testing environment for React components.
 */

import { vi } from 'vitest';
import '@testing-library/jest-dom';

// Mock browser APIs not available in jsdom
global.HTMLElement.prototype.scrollIntoView = vi.fn();

// Mock fetch if not already defined
if (!global.fetch) {
  global.fetch = vi.fn();
}
