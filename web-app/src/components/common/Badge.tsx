import React from 'react'
import clsx from 'clsx'
import { BadgeProps } from '@/types'

export const Badge: React.FC<BadgeProps> = ({ variant = 'neutral', size = 'md', children, className, icon }) => {
  const variantStyles = {
    success: 'bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200',
    error: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
    warning: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
    info: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
    neutral: 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200',
  }

  const sizeStyles = {
    sm: 'px-2 py-1 text-xs font-medium',
    md: 'px-3 py-1.5 text-sm font-medium',
  }

  return (
    <span
      className={clsx(
        'inline-flex items-center gap-1.5 rounded-full',
        variantStyles[variant],
        sizeStyles[size],
        className
      )}
    >
      {icon && <span className="shrink-0">{icon}</span>}
      {children}
    </span>
  )
}

export default Badge
