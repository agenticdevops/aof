import React from 'react'
import clsx from 'clsx'

interface WizardProgressProps {
  currentStep: 1 | 2 | 3 | 4
  completedSteps?: Set<number>
}

const steps = [
  { number: 1, label: 'Project Setup' },
  { number: 2, label: 'Agent Config' },
  { number: 3, label: 'Platforms' },
  { number: 4, label: 'Review' },
]

export const WizardProgress: React.FC<WizardProgressProps> = ({ currentStep, completedSteps = new Set() }) => {
  return (
    <div className="flex flex-col gap-6">
      {/* Progress indicator */}
      <div className="flex items-center gap-3">
        {steps.map((step, index) => (
          <React.Fragment key={step.number}>
            {/* Step circle */}
            <div
              className={clsx(
                'w-10 h-10 rounded-full flex items-center justify-center font-semibold transition-all',
                {
                  'bg-emerald-500 text-white': step.number === currentStep,
                  'bg-emerald-100 text-emerald-700 dark:bg-emerald-900 dark:text-emerald-200':
                    completedSteps.has(step.number),
                  'bg-gray-200 text-gray-500 dark:bg-gray-700 dark:text-gray-400':
                    step.number > currentStep && !completedSteps.has(step.number),
                }
              )}
            >
              {completedSteps.has(step.number) ? (
                <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" />
                </svg>
              ) : (
                step.number
              )}
            </div>

            {/* Connector */}
            {index < steps.length - 1 && (
              <div
                className={clsx('flex-1 h-1 transition-colors', {
                  'bg-emerald-500': step.number < currentStep,
                  'bg-gray-200 dark:bg-gray-700': step.number >= currentStep,
                })}
              />
            )}
          </React.Fragment>
        ))}
      </div>

      {/* Step labels */}
      <div className="flex justify-between text-xs">
        {steps.map((step) => (
          <span
            key={step.number}
            className={clsx('font-medium', {
              'text-emerald-600 dark:text-emerald-400': step.number === currentStep,
              'text-gray-500 dark:text-gray-400': step.number !== currentStep,
            })}
          >
            {step.label}
          </span>
        ))}
      </div>
    </div>
  )
}

export default WizardProgress
