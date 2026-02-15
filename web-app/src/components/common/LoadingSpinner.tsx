import React from 'react'

type SpinnerSize = 'sm' | 'md' | 'lg'

interface LoadingSpinnerProps {
  size?: SpinnerSize
  fullPage?: boolean
  text?: string
}

const sizeClasses: Record<SpinnerSize, string> = {
  sm: 'w-6 h-6',
  md: 'w-10 h-10',
  lg: 'w-16 h-16',
}

export const LoadingSpinner: React.FC<LoadingSpinnerProps> = ({ size = 'md', fullPage = false, text }) => {
  const spinner = (
    <div className="flex flex-col items-center justify-center gap-4">
      <svg className={`${sizeClasses[size]} animate-spin text-sky-400 dark:text-sky-500`} xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
        <path
          className="opacity-75"
          fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
        />
      </svg>
      {text && <p className="text-gray-600 dark:text-gray-400 text-sm font-medium">{text}</p>}
    </div>
  )

  if (fullPage) {
    return (
      <div className="fixed inset-0 bg-white dark:bg-gray-900 flex items-center justify-center z-50">
        {spinner}
      </div>
    )
  }

  return spinner
}

export default LoadingSpinner
