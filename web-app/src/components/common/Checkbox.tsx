import React from 'react'
import { Check } from 'lucide-react'

interface CheckboxProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string
  error?: string
}

export const Checkbox = React.forwardRef<HTMLInputElement, CheckboxProps>(
  ({ label, error, className = '', id, ...props }, ref) => {
    const checkboxId = id || `checkbox-${Math.random().toString(36).substr(2, 9)}`

    return (
      <div>
        <div className="flex items-center">
          <div className="relative">
            <input
              ref={ref}
              id={checkboxId}
              type="checkbox"
              className="sr-only"
              {...props}
            />
            <div
              className={`
                w-5 h-5 rounded border-2 transition-all duration-200
                flex items-center justify-center cursor-pointer
                ${
                  props.checked
                    ? 'border-sky-400 bg-sky-400 dark:border-sky-500 dark:bg-sky-500'
                    : 'border-gray-300 dark:border-gray-600 hover:border-sky-300 dark:hover:border-sky-400'
                }
                ${props.disabled ? 'opacity-50 cursor-not-allowed' : ''}
                focus:outline-none focus:ring-2 focus:ring-sky-400 dark:focus:ring-sky-500
              `}
            >
              {props.checked && <Check className="w-3 h-3 text-white" />}
            </div>
          </div>
          {label && (
            <label htmlFor={checkboxId} className="ml-3 text-sm font-medium text-gray-700 dark:text-gray-300 cursor-pointer">
              {label}
            </label>
          )}
        </div>
        {error && <p className="text-red-500 text-sm mt-1">{error}</p>}
      </div>
    )
  }
)

Checkbox.displayName = 'Checkbox'
export default Checkbox
