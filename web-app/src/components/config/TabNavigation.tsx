import React from 'react'

type TabType = 'agents' | 'tools' | 'platforms'

interface TabNavigationProps {
  activeTab: TabType
  onTabChange: (tab: TabType) => void
}

const tabs: { value: TabType; label: string }[] = [
  { value: 'agents', label: 'Agents' },
  { value: 'tools', label: 'Tools' },
  { value: 'platforms', label: 'Platforms' },
]

export const TabNavigation: React.FC<TabNavigationProps> = ({ activeTab, onTabChange }) => {
  return (
    <div className="flex gap-2 border-b border-gray-200 dark:border-gray-700">
      {tabs.map((tab) => (
        <button
          key={tab.value}
          onClick={() => onTabChange(tab.value)}
          className={`px-4 py-3 font-medium transition-colors border-b-2 ${
            activeTab === tab.value
              ? 'border-sky-400 text-sky-600 dark:border-sky-500 dark:text-sky-400'
              : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-300'
          }`}
        >
          {tab.label}
        </button>
      ))}
    </div>
  )
}

export default TabNavigation
