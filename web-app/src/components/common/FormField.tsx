import React from 'react'
import clsx from 'clsx'
import { FormFieldProps } from '@/types'

export const FormField: React.FC<FormFieldProps> = ({ label, error, required, helperText, children, className }) => {
  return (
    <div className={clsx('flex flex-col gap-1.5', className)}>
      {label && (
        <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
          {label}
          {required && <span className="text-red-500 ml-0.5">*</span>}
        </label>
      )}

      {children}

      {error && <p className="text-sm text-red-500">{error}</p>}
      {helperText && !error && <p className="text-sm text-gray-500 dark:text-gray-400">{helperText}</p>}
    </div>
  )
}

export default FormField
