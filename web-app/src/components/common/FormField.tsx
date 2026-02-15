import React from 'react'

interface FormFieldProps {
  label?: string
  error?: string
  helperText?: string
  required?: boolean
  children: React.ReactNode
}

export const FormField: React.FC<FormFieldProps> = ({ label, error, helperText, required, children }) => {
  return (
    <div>
      {label && (
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          {label}
          {required && <span className="text-red-500 ml-1">*</span>}
        </label>
      )}
      {children}
      {error && <p className="text-red-500 text-sm mt-1">{error}</p>}
      {helperText && <p className="text-gray-500 dark:text-gray-400 text-sm mt-1">{helperText}</p>}
    </div>
  )
}

export default FormField
