import React from 'react'
import { ChevronDown } from 'lucide-react'

interface SelectOption {
  value: string
  label: string
  disabled?: boolean
}

interface SelectGroup {
  label: string
  options: SelectOption[]
}

interface SelectProps extends Omit<React.SelectHTMLAttributes<HTMLSelectElement>, 'children'> {
  label?: string
  error?: string
  helperText?: string
  fullWidth?: boolean
  options?: SelectOption[] | SelectGroup[]
  placeholder?: string
}

const isGroup = (item: SelectOption | SelectGroup): item is SelectGroup => {
  return 'options' in item
}

export const Select = React.forwardRef<HTMLSelectElement, SelectProps>(
  ({ label, error, helperText, fullWidth = false, options = [], placeholder, className = '', ...props }, ref) => {
    return (
      <div className={fullWidth ? 'w-full' : ''}>
        {label && (
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {label}
            {props.required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}
        <div className="relative">
          <select
            ref={ref}
            className={`
              w-full px-4 py-2 pr-10 rounded-lg border appearance-none transition-colors
              bg-white dark:bg-gray-800 text-gray-900 dark:text-white
              focus:outline-none focus:ring-2 focus:ring-sky-400 dark:focus:ring-sky-500
              focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed
              ${error ? 'border-red-500 focus:ring-red-400' : 'border-gray-300 dark:border-gray-600'}
              ${className}
            `}
            {...props}
          >
            {placeholder && <option value="">{placeholder}</option>}
            {options.map((item, index) =>
              isGroup(item) ? (
                <optgroup key={index} label={item.label}>
                  {item.options.map((opt, optIndex) => (
                    <option key={optIndex} value={opt.value} disabled={opt.disabled}>
                      {opt.label}
                    </option>
                  ))}
                </optgroup>
              ) : (
                <option key={index} value={item.value} disabled={item.disabled}>
                  {item.label}
                </option>
              )
            )}
          </select>
          <ChevronDown className="absolute right-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400 pointer-events-none" />
        </div>
        {error && <p className="text-red-500 text-sm mt-1">{error}</p>}
        {helperText && <p className="text-gray-500 dark:text-gray-400 text-sm mt-1">{helperText}</p>}
      </div>
    )
  }
)

Select.displayName = 'Select'
export default Select
