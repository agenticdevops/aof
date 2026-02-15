import React from 'react'
import { CheckCircle, Circle } from 'lucide-react'

interface WizardProgressProps {
  currentStep: 1 | 2 | 3 | 4
  completedSteps: number[]
}

const steps = [
  { number: 1, title: 'Project Setup' },
  { number: 2, title: 'Agent Configuration' },
  { number: 3, title: 'Platform Connections' },
  { number: 4, title: 'Review & Launch' },
]

export const WizardProgress: React.FC<WizardProgressProps> = ({ currentStep, completedSteps }) => {
  return (
    <div className="space-y-6">
      {/* Progress Bar */}
      <div className="flex items-center justify-between">
        {steps.map((step, index) => (
          <React.Fragment key={step.number}>
            {/* Step Circle */}
            <div className="flex flex-col items-center flex-1">
              <div
                className={`w-12 h-12 rounded-full flex items-center justify-center font-bold text-lg transition-all duration-200 ${
                  step.number === currentStep
                    ? 'bg-sky-400 text-white dark:bg-sky-500'
                    : completedSteps.includes(step.number)
                    ? 'bg-green-500 text-white dark:bg-green-600'
                    : 'bg-gray-200 text-gray-600 dark:bg-gray-700 dark:text-gray-400'
                }`}
              >
                {completedSteps.includes(step.number) ? (
                  <CheckCircle className="w-6 h-6" />
                ) : (
                  <span>{step.number}</span>
                )}
              </div>
              <p className="text-xs sm:text-sm font-medium text-gray-700 dark:text-gray-300 mt-2 text-center">
                {step.title}
              </p>
            </div>

            {/* Connector Line */}
            {index < steps.length - 1 && (
              <div
                className={`h-1 flex-1 mx-2 rounded transition-all duration-200 ${
                  completedSteps.includes(step.number)
                    ? 'bg-green-500 dark:bg-green-600'
                    : 'bg-gray-300 dark:bg-gray-700'
                }`}
              />
            )}
          </React.Fragment>
        ))}
      </div>
    </div>
  )
}

export default WizardProgress
