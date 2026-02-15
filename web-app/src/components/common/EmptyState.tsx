import React from 'react'
import clsx from 'clsx'

interface EmptyStateProps {
  icon?: React.ReactNode
  title: string
  description?: string
  action?: React.ReactNode
  className?: string
}

export const EmptyState: React.FC<EmptyStateProps> = ({ icon, title, description, action, className }) => {
  return (
    <div
      className={clsx(
        'flex flex-col items-center justify-center gap-4 py-12 px-4 text-center',
        'bg-gray-50 dark:bg-gray-900 rounded-lg border border-dashed border-gray-200 dark:border-gray-700',
        className
      )}
    >
      {icon && <div className="w-16 h-16 text-gray-300 dark:text-gray-600">{icon}</div>}

      <div className="flex flex-col gap-2">
        <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">{title}</h3>
        {description && <p className="text-sm text-gray-500 dark:text-gray-400">{description}</p>}
      </div>

      {action && <div className="mt-4">{action}</div>}
    </div>
  )
}

export default EmptyState
