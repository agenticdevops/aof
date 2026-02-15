import React from 'react'
import { useAppDispatch } from '@/store'
import { setNavigation, setFirstVisit } from '@/store'
import Button from '@/components/common/Button'
import Card from '@/components/common/Card'
import { Activity, Users, MessageSquare, ArrowRight } from 'lucide-react'

export const WelcomePage: React.FC = () => {
  const dispatch = useAppDispatch()

  const handleStartSetup = () => {
    dispatch(setFirstVisit(false))
    dispatch(setNavigation('wizard'))
  }

  const features = [
    {
      icon: Activity,
      title: 'Real-time Monitoring',
      description: 'Watch your agents coordinate and complete tasks with live updates',
    },
    {
      icon: Users,
      title: 'Agent Personas',
      description: 'Create specialized agents with unique skills and personalities',
    },
    {
      icon: MessageSquare,
      title: 'Squad Communication',
      description: 'Enable agents to chat, share context, and collaborate seamlessly',
    },
  ]

  return (
    <div className="min-h-screen bg-gradient-to-br from-sky-50 to-blue-50 dark:from-gray-900 dark:to-gray-800">
      {/* Hero Section */}
      <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-20 sm:py-32">
        <div className="text-center mb-16">
          <h1 className="text-5xl sm:text-6xl font-bold text-gray-900 dark:text-gray-100 mb-6 leading-tight">
            Meet Your <span className="text-sky-400 dark:text-sky-300">Agent Squad</span>
          </h1>
          <p className="text-xl text-gray-600 dark:text-gray-400 max-w-2xl mx-auto mb-8">
            Build AI agents that work together. Configure agents, connect platforms, and watch them collaborate in real-time to accomplish your goals.
          </p>
          <Button
            size="lg"
            onClick={handleStartSetup}
            icon={<ArrowRight className="w-5 h-5" />}
            iconPosition="right"
          >
            Begin Setup
          </Button>
        </div>

        {/* Feature Cards */}
        <div className="grid md:grid-cols-3 gap-6 mb-16">
          {features.map((feature) => {
            const Icon = feature.icon
            return (
              <Card key={feature.title} elevation="lifted" hoverable>
                <div className="text-center">
                  <div className="inline-flex items-center justify-center w-12 h-12 rounded-lg bg-sky-100 dark:bg-sky-900 mb-4">
                    <Icon className="w-6 h-6 text-sky-600 dark:text-sky-300" />
                  </div>
                  <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">{feature.title}</h3>
                  <p className="text-gray-600 dark:text-gray-400">{feature.description}</p>
                </div>
              </Card>
            )
          })}
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
