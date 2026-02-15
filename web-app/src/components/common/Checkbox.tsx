import React from 'react'
import clsx from 'clsx'

interface CheckboxProps {
  label?: string
  checked?: boolean
  onChange?: (checked: boolean) => void
  disabled?: boolean
  className?: string
  error?: string
}

export const Checkbox: React.FC<CheckboxProps> = ({
  label,
  checked = false,
  onChange,
  disabled = false,
  className,
  error,
}) => {
  return (
    <div className="flex flex-col gap-1">
      <label className={clsx('flex items-center gap-2 cursor-pointer', { 'opacity-50 cursor-not-allowed': disabled })}>
        <input
          type="checkbox"
          checked={checked}
          onChange={(e) => onChange?.(e.target.checked)}
          disabled={disabled}
          className={clsx(
            'w-4 h-4 rounded border border-gray-300 dark:border-gray-600',
            'bg-white dark:bg-gray-800 text-sky-400',
            'focus:ring-2 focus:ring-sky-400 focus:border-transparent',
            'cursor-pointer transition-colors',
            className
          )}
        />
        {label && <span className="text-sm text-gray-700 dark:text-gray-300">{label}</span>}
      </label>
      {error && <p className="text-sm text-red-500">{error}</p>}
    </div>
  )
}

export default Checkbox
