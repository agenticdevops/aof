import React from 'react'
import { useAppDispatch, useAppSelector, setStep, markStepCompleted } from '@/store'
import WizardProgress from '@/components/onboarding/WizardProgress'
import StepWelcome from '@/components/onboarding/StepWelcome'
import StepAgentSetup from '@/components/onboarding/StepAgentSetup'
import StepPlatformConfig from '@/components/onboarding/StepPlatformConfig'
import StepReview from '@/components/onboarding/StepReview'
import Card from '@/components/common/Card'

export const OnboardingWizard: React.FC = () => {
  const dispatch = useAppDispatch()
  const { currentStep, completedSteps } = useAppSelector((state) => state.onboarding)

  const handleNext = () => {
    dispatch(markStepCompleted(currentStep))
    dispatch(setStep((currentStep + 1) as 1 | 2 | 3 | 4))
  }

  const handleBack = () => {
    dispatch(setStep((currentStep - 1) as 1 | 2 | 3 | 4))
  }

  const renderStep = () => {
    switch (currentStep) {
      case 1:
        return <StepWelcome onNext={handleNext} />
      case 2:
        return <StepAgentSetup onBack={handleBack} onNext={handleNext} />
      case 3:
        return <StepPlatformConfig onBack={handleBack} onNext={handleNext} />
      case 4:
        return <StepReview onBack={handleBack} />
      default:
        return null
    }
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-8 px-4 sm:px-6 lg:px-8">
      <div className="max-w-3xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">AOF Setup Wizard</h1>
          <p className="text-gray-600 dark:text-gray-400 mt-2">Set up your first agent in minutes</p>
        </div>

        {/* Progress Indicator */}
        <div className="mb-8">
          <WizardProgress currentStep={currentStep} completedSteps={completedSteps} />
        </div>

        {/* Wizard Card */}
        <Card elevation="lifted" className="mb-8">
          {renderStep()}
        </Card>

        {/* Footer Info */}
        <div className="text-center text-sm text-gray-500 dark:text-gray-400">
          <p>Step {currentStep} of 4</p>
        </div>
      </div>
    </div>
  )
}

export default OnboardingWizard
