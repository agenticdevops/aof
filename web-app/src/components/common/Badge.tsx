import React from 'react'

type BadgeVariant = 'success' | 'error' | 'warning' | 'info' | 'neutral'
type BadgeSize = 'sm' | 'md'

interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  variant?: BadgeVariant
  size?: BadgeSize
  icon?: React.ReactNode
  children: React.ReactNode
}

const variantStyles: Record<BadgeVariant, string> = {
  success: 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300',
  error: 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300',
  warning: 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-300',
  info: 'bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300',
  neutral: 'bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-300',
}

const sizeStyles: Record<BadgeSize, string> = {
  sm: 'px-2 py-1 text-xs',
  md: 'px-3 py-1.5 text-sm',
}

export const Badge = React.forwardRef<HTMLSpanElement, BadgeProps>(
  ({ variant = 'neutral', size = 'md', icon, children, className = '', ...props }, ref) => {
    return (
      <span
        ref={ref}
        className={`
          inline-flex items-center gap-1.5 font-medium rounded-full
          ${variantStyles[variant]}
          ${sizeStyles[size]}
          ${className}
        `}
        {...props}
      >
        {icon}
        {children}
      </span>
    )
  }
)

Badge.displayName = 'Badge'
export default Badge
