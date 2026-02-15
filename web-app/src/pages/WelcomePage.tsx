import React from 'react'
import { useAppDispatch } from '@/store'
import { setNavigation, setFirstVisit } from '@/store'
import Button from '@/components/common/Button'
import Card from '@/components/common/Card'

export const WelcomePage: React.FC = () => {
  const dispatch = useAppDispatch()

  const handleStartSetup = () => {
    dispatch(setFirstVisit(false))
    dispatch(setNavigation('wizard'))
  }

  const features = [
    {
      icon: '📊',
      title: 'Real-time Monitoring',
      description: 'Watch your agents coordinate and complete tasks with live updates',
    },
    {
      icon: '🤝',
      title: 'Agent Personas',
      description: 'Create specialized agents with unique skills and personalities',
    },
    {
      icon: '💬',
      title: 'Squad Communication',
      description: 'Enable agents to chat, share context, and collaborate seamlessly',
    },
  ]

  return (
    <div className="min-h-screen bg-gradient-to-br from-emerald-50 to-blue-50 dark:from-gray-900 dark:to-gray-800">
      {/* Hero Section */}
      <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-20 sm:py-32">
        <div className="text-center mb-16">
          <h1 className="text-5xl sm:text-6xl font-bold text-gray-900 dark:text-gray-100 mb-6 leading-tight">
            Meet Your <span className="text-emerald-600 dark:text-emerald-400">Agent Squad</span>
          </h1>
          <p className="text-xl text-gray-600 dark:text-gray-400 max-w-2xl mx-auto mb-8">
            Build AI agents that work together. Configure agents, connect platforms, and watch them collaborate in real-time to accomplish your goals.
          </p>
          <Button
            size="lg"
            onClick={handleStartSetup}
            icon={
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
            }
            iconPosition="right"
          >
            Begin Setup
          </Button>
        </div>

        {/* Feature Cards */}
        <div className="grid md:grid-cols-3 gap-6 mb-16">
          {features.map((feature) => (
            <Card key={feature.title} elevation="lifted" hoverable>
              <div className="text-center">
                <div className="text-5xl mb-4">{feature.icon}</div>
                <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">{feature.title}</h3>
                <p className="text-gray-600 dark:text-gray-400">{feature.description}</p>
              </div>
            </Card>
          ))}
        </div>

        {/* Info Section */}
        <Card elevation="focused" className="max-w-2xl mx-auto">
          <div className="text-center">
            <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-4">Ready to automate?</h2>
            <p className="text-gray-600 dark:text-gray-400 mb-6">
              In just 4 simple steps, you'll have your first agent running and ready to handle your tasks.
            </p>
            <div className="flex justify-center gap-4">
              <Button variant="secondary">Learn More</Button>
              <Button onClick={handleStartSetup}>Start Now</Button>
            </div>
          </div>
        </Card>
      </div>
    </div>
  )
}

export default WelcomePage
