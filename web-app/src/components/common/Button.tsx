import React from 'react'
import clsx from 'clsx'
import { ButtonProps } from '@/types'

export const Button: React.FC<ButtonProps> = ({
  variant = 'primary',
  size = 'md',
  loading = false,
  disabled = false,
  fullWidth = false,
  children,
  onClick,
  className,
  type = 'button',
  icon,
  iconPosition = 'left',
}) => {
  const isDisabled = disabled || loading

  const baseStyles =
    'inline-flex items-center justify-center gap-2 font-medium rounded-lg transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 dark:focus:ring-offset-gray-900'

  const variantStyles = {
    primary:
      'bg-sky-400/80 text-white hover:bg-sky-500 active:bg-sky-600 focus:ring-sky-300 dark:bg-sky-500 dark:hover:bg-sky-600 disabled:bg-gray-300 disabled:text-gray-500',
    secondary:
      'bg-gray-100 text-gray-900 hover:bg-gray-200 active:bg-gray-300 focus:ring-gray-400 dark:bg-gray-700 dark:text-gray-100 dark:hover:bg-gray-600 disabled:bg-gray-200 disabled:text-gray-400',
    ghost:
      'text-gray-600 hover:bg-gray-100 active:bg-gray-200 focus:ring-gray-300 dark:text-gray-300 dark:hover:bg-gray-700 disabled:text-gray-400',
    danger:
      'bg-red-500 text-white hover:bg-red-600 active:bg-red-700 focus:ring-red-400 dark:hover:bg-red-600 disabled:bg-gray-300 disabled:text-gray-500',
  }

  const sizeStyles = {
    sm: 'px-3 py-1.5 text-sm',
    md: 'px-4 py-2 text-base',
    lg: 'px-6 py-3 text-lg',
  }

  return (
    <button
      type={type}
      disabled={isDisabled}
      onClick={onClick}
      className={clsx(
        baseStyles,
        variantStyles[variant],
        sizeStyles[size],
        { 'opacity-50 cursor-not-allowed': isDisabled },
        { 'w-full': fullWidth },
        className
      )}
    >
      {icon && iconPosition === 'left' && <span className="shrink-0">{icon}</span>}
      {loading ? <span className="animate-spin">⏳</span> : children}
      {icon && iconPosition === 'right' && <span className="shrink-0">{icon}</span>}
    </button>
  )
}

export default Button
