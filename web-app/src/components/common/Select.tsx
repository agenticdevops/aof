import React from 'react'
import clsx from 'clsx'
import { SelectProps } from '@/types'

export const Select: React.FC<SelectProps> = ({
  label,
  error,
  disabled = false,
  placeholder = 'Select an option',
  value,
  onChange,
  options,
  required = false,
  helperText,
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

      <select
        disabled={disabled}
        value={value || ''}
        onChange={(e) => onChange?.(e.target.value)}
        className={clsx(
          'w-full px-4 py-2 border rounded-lg transition-colors duration-200',
          'bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100',
          'focus:outline-none focus:ring-2 focus:ring-sky-400 focus:border-transparent',
          'appearance-none cursor-pointer',
          {
            'border-red-500 focus:ring-red-400': error,
            'border-gray-300 dark:border-gray-600': !error,
            'opacity-50 cursor-not-allowed bg-gray-50 dark:bg-gray-900': disabled,
          }
        )}
      >
        <option value="">{placeholder}</option>
        {options.map((option) => (
          <option key={option.value} value={option.value} disabled={option.disabled}>
            {option.label}
          </option>
        ))}
      </select>

      <svg
        className="absolute right-3 top-[38px] w-5 h-5 text-gray-400 pointer-events-none"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
      </svg>

      {error && <p className="mt-1 text-sm text-red-500">{error}</p>}
      {helperText && !error && <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">{helperText}</p>}
    </div>
  )
}

export default Select
