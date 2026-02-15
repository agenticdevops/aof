import React from 'react'
import clsx from 'clsx'
import { CardProps } from '@/types'

export const Card: React.FC<CardProps> = ({
  children,
  className,
  elevation = 'lifted',
  clickable = false,
  onClick,
  hoverable = false,
}) => {
  const elevationStyles = {
    flat: 'bg-white dark:bg-gray-800',
    lifted: 'bg-white dark:bg-gray-800 shadow-md',
    focused: 'bg-white dark:bg-gray-800 shadow-lg ring-1 ring-gray-200 dark:ring-gray-700',
  }

  return (
    <div
      onClick={onClick}
      className={clsx(
        'rounded-lg border border-gray-200 dark:border-gray-700 p-6 transition-all duration-200',
        elevationStyles[elevation],
        {
          'cursor-pointer hover:shadow-lg': clickable,
          'hover:shadow-lg hover:border-gray-300 dark:hover:border-gray-600': hoverable,
        },
        className
      )}
    >
      {children}
    </div>
  )
}

export default Card
