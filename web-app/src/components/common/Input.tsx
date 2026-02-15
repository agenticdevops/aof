import React from 'react'
import clsx from 'clsx'
import { InputProps } from '@/types'

export const Input: React.FC<InputProps> = ({
  label,
  error,
  disabled = false,
  placeholder,
  value,
  onChange,
  type = 'text',
  required = false,
  helperText,
  icon,
  className,
  fullWidth = false,
}) => {
  return (
    <div className={clsx({ 'w-full': fullWidth })}>
      {label && (
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">
          {label}
          {required && <span className="text-red-500 ml-0.5">*</span>}
        </label>
      )}

      <div className="relative">
        {icon && <span className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400">{icon}</span>}
        <input
          type={type}
          disabled={disabled}
          placeholder={placeholder}
          value={value}
          onChange={onChange}
          className={clsx(
            'w-full px-4 py-2 border rounded-lg transition-colors duration-200',
            'bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100',
            'placeholder-gray-400 dark:placeholder-gray-500',
            'focus:outline-none focus:ring-2 focus:ring-emerald-400 focus:border-transparent',
            {
              'pl-10': icon,
              'border-red-500 focus:ring-red-400': error,
              'border-gray-300 dark:border-gray-600': !error,
              'opacity-50 cursor-not-allowed bg-gray-50 dark:bg-gray-900': disabled,
            },
            className
          )}
        />
      </div>

      {error && <p className="mt-1 text-sm text-red-500">{error}</p>}
      {helperText && !error && <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">{helperText}</p>}
    </div>
  )
}

export default Input
