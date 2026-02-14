/**
 * Skeleton loader component for loading states.
 * Provides consistent placeholder animations across the app.
 */

import React from 'react';

/**
 * Component props.
 */
export interface SkeletonProps {
  /** Width (CSS value: px, %, rem, etc.) */
  width?: string;

  /** Height (CSS value) */
  height?: string;

  /** Variant type */
  variant?: 'text' | 'circular' | 'rectangular';

  /** Optional className for styling */
  className?: string;
}

/**
 * Skeleton component.
 *
 * @example
 * ```tsx
 * <Skeleton width="100%" height="20px" variant="text" />
 * <Skeleton width="40px" height="40px" variant="circular" />
 * <Skeleton width="200px" height="150px" variant="rectangular" />
 * ```
 */
export function Skeleton({
  width = '100%',
  height = '20px',
  variant = 'rectangular',
  className = '',
}: SkeletonProps): React.ReactElement {
  const variantClasses = {
    text: 'rounded',
    circular: 'rounded-full',
    rectangular: 'rounded-lg',
  };

  return (
    <div
      className={`bg-gray-300 dark:bg-gray-600 animate-pulse ${variantClasses[variant]} ${className}`}
      style={{ width, height }}
      aria-hidden="true"
    />
  );
}
