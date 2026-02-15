import React from 'react'
import clsx from 'clsx'

interface LoadingSpinnerProps {
  size?: 'sm' | 'md' | 'lg'
  fullPage?: boolean
  text?: string
}

export const LoadingSpinner: React.FC<LoadingSpinnerProps> = ({ size = 'md', fullPage = false, text }) => {
  const sizeStyles = {
    sm: 'w-6 h-6',
    md: 'w-10 h-10',
    lg: 'w-16 h-16',
  }

  const content = (
    <div className="flex flex-col items-center gap-3">
      <div className={clsx('animate-spin text-emerald-500', sizeStyles[size])}>
        <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="2" fill="none" opacity="0.3" />
          <path d="M12 2a10 10 0 0110 10" strokeLinecap="round" strokeWidth="2" />
        </svg>
      </div>
      {text && <p className="text-sm text-gray-600 dark:text-gray-400">{text}</p>}
    </div>
  )

  if (fullPage) {
    return (
      <div className="fixed inset-0 flex items-center justify-center bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm z-50">
        {content}
      </div>
    )
  }

  return <div className="flex items-center justify-center">{content}</div>
}

export default LoadingSpinner
