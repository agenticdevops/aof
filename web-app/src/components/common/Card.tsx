import React from 'react'

type CardElevation = 'flat' | 'lifted' | 'focused'

interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  elevation?: CardElevation
  hoverable?: boolean
  clickable?: boolean
}

const elevationStyles: Record<CardElevation, string> = {
  flat: 'border border-gray-200 dark:border-gray-700',
  lifted: 'border border-gray-200 dark:border-gray-700 shadow-md',
  focused: 'border-2 border-sky-400 dark:border-sky-500 shadow-lg',
}

export const Card = React.forwardRef<HTMLDivElement, CardProps>(
  ({ elevation = 'flat', hoverable = false, clickable = false, className = '', ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={`
          p-6 rounded-lg bg-white dark:bg-gray-800
          transition-all duration-200
          ${elevationStyles[elevation]}
          ${hoverable ? 'hover:shadow-lg dark:hover:shadow-lg' : ''}
          ${clickable ? 'cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700' : ''}
          ${className}
        `}
        {...props}
      />
    )
  }
)

Card.displayName = 'Card'
export default Card
