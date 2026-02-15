import React from 'react'
import { Loader2 } from 'lucide-react'

type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger'
type ButtonSize = 'sm' | 'md' | 'lg'

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant
  size?: ButtonSize
  loading?: boolean
  fullWidth?: boolean
  icon?: React.ReactNode
  iconPosition?: 'left' | 'right'
}

const variantStyles: Record<ButtonVariant, string> = {
  primary: 'bg-sky-400 hover:bg-sky-500 text-white disabled:bg-sky-300 dark:bg-sky-500 dark:hover:bg-sky-600',
  secondary: 'bg-gray-200 hover:bg-gray-300 text-gray-900 disabled:bg-gray-100 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-white',
  ghost: 'hover:bg-gray-100 text-gray-900 dark:hover:bg-gray-800 dark:text-white',
  danger: 'bg-red-500 hover:bg-red-600 text-white disabled:bg-red-400 dark:bg-red-600 dark:hover:bg-red-700',
}

const sizeStyles: Record<ButtonSize, string> = {
  sm: 'px-3 py-1.5 text-sm',
  md: 'px-4 py-2 text-base',
  lg: 'px-6 py-3 text-lg',
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = 'primary',
      size = 'md',
      loading = false,
      fullWidth = false,
      disabled = false,
      icon,
      iconPosition = 'left',
      className = '',
      children,
      ...props
    },
    ref
  ) => {
    return (
      <button
        ref={ref}
        disabled={disabled || loading}
        className={`
          inline-flex items-center justify-center gap-2 font-medium rounded-lg
          transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-sky-400
          dark:focus:ring-sky-500 focus:ring-offset-2 dark:focus:ring-offset-gray-900
          disabled:opacity-50 disabled:cursor-not-allowed
          ${variantStyles[variant]}
          ${sizeStyles[size]}
          ${fullWidth ? 'w-full' : ''}
          ${className}
        `}
        {...props}
      >
        {loading && <Loader2 className="w-4 h-4 animate-spin" />}
        {icon && iconPosition === 'left' && !loading && <span>{icon}</span>}
        {children}
        {icon && iconPosition === 'right' && !loading && <span>{icon}</span>}
      </button>
    )
  }
)

Button.displayName = 'Button'
export default Button
