import React from 'react'
import clsx from 'clsx'

interface TabNavigationProps {
  activeTab: 'agents' | 'tools' | 'platforms'
  onTabChange: (tab: 'agents' | 'tools' | 'platforms') => void
}

const tabs = [
  { id: 'agents', label: 'Agents', icon: '🤖' },
  { id: 'tools', label: 'Tools', icon: '🔧' },
  { id: 'platforms', label: 'Platforms', icon: '🌐' },
]

export const TabNavigation: React.FC<TabNavigationProps> = ({ activeTab, onTabChange }) => {
  return (
    <div className="flex gap-4 border-b border-gray-200 dark:border-gray-700">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          onClick={() => onTabChange(tab.id as any)}
          className={clsx(
            'px-4 py-4 font-medium transition-colors border-b-2 -mb-[2px]',
            {
              'border-sky-400/80 text-sky-600 dark:text-sky-400 dark:border-sky-500': activeTab === tab.id,
              'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-300':
                activeTab !== tab.id,
            }
          )}
        >
          <span className="mr-2">{tab.icon}</span>
          {tab.label}
        </button>
      ))}
    </div>
  )
}

export default TabNavigation
