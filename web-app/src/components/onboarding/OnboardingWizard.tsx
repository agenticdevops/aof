import React, { useEffect } from 'react'
import { useAppDispatch, useAppSelector, setCurrentStep, submitWizard } from '@/store'
import Button from '@/components/common/Button'
import Card from '@/components/common/Card'
import Alert from '@/components/common/Alert'
import StepChannels from './StepChannels'
import StepAIModel from './StepAIModel'
import StepTools from './StepTools'
import StepReview from './StepReview'

/**
 * Main wizard component for the 3-step Xops onboarding flow
 * Steps: 1) Channels → 2) AI Model → 3) Tools → 4) Review & Launch
 */
export const OnboardingWizard: React.FC = () => {
  const dispatch = useAppDispatch()
  const { currentStep, selectedChannels, selectedModel, selectedTools, loading, error } = useAppSelector(
    (state) => state.onboarding
  )

  // Validate current step before allowing progression
  const isCurrentStepValid = (): boolean => {
    switch (currentStep) {
      case 1:
        // Step 1: At least one channel must be selected
        return selectedChannels.length > 0
      case 2:
        // Step 2: Model must be selected (always true, has default)
        return true
      case 3:
        // Step 3: No tools required, but good to have at least one
        return true
      case 4:
        // Step 4: All data required
        return selectedChannels.length > 0 && selectedModel
      default:
        return false
    }
  }

  const handleNext = async () => {
    if (!isCurrentStepValid()) {
      return
    }

    if (currentStep < 4) {
      dispatch(setCurrentStep(currentStep + 1))
    }
  }

  const handleBack = () => {
    if (currentStep > 1) {
      dispatch(setCurrentStep(currentStep - 1))
    }
  }

  const handleSubmit = async () => {
    const result = await dispatch(submitWizard())
    if (submitWizard.fulfilled.match(result)) {
      // Xops created successfully, navigate to dashboard
      window.location.hash = '#/dashboard'
    }
  }

  // Determine if Next button should be disabled
  const isNextDisabled = (): boolean => {
    if (loading) return true
    if (currentStep === 1 && selectedChannels.length === 0) return true
    if (currentStep === 4) return false // Submit button, not Next
    return false
  }

  // Determine button text
  const getButtonText = (): string => {
    if (currentStep === 4) {
      return loading ? 'Launching...' : 'Launch Xops'
    }
    return loading ? 'Continuing...' : 'Next'
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-8 px-4 sm:px-6 lg:px-8">
      <div className="max-w-3xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
            Welcome to Xops
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            Let's set up your orchestrator agent in just a few steps
          </p>
        </div>

        {/* Progress Indicator */}
        <div className="mb-8">
          <div className="flex items-center justify-between mb-2">
            <h2 className="text-sm font-medium text-gray-900 dark:text-white">
              Step {currentStep} of 4
            </h2>
            <span className="text-sm text-gray-500 dark:text-gray-400">
              {getStepTitle(currentStep)}
            </span>
          </div>
          <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
            <div
              className="bg-blue-600 h-2 rounded-full transition-all duration-300"
              style={{ width: `${(currentStep / 4) * 100}%` }}
            />
          </div>
        </div>

        {/* Error Alert */}
        {error && (
          <div className="mb-6">
            <Alert variant="error" title="Configuration Error">
              {error}
            </Alert>
          </div>
        )}

        {/* Wizard Card */}
        <Card elevation="lifted" className="mb-8 p-6">
          {currentStep === 1 && <StepChannels />}
          {currentStep === 2 && <StepAIModel />}
          {currentStep === 3 && <StepTools />}
          {currentStep === 4 && <StepReview />}
        </Card>

        {/* Navigation Controls */}
        <div className="flex gap-3 justify-between">
          <Button
            variant="secondary"
            onClick={handleBack}
            disabled={currentStep === 1 || loading}
            className="flex-1"
          >
            Back
          </Button>

          <Button
            variant="primary"
            onClick={currentStep === 4 ? handleSubmit : handleNext}
            disabled={isNextDisabled()}
            loading={loading}
            className="flex-1"
          >
            {getButtonText()}
          </Button>
        </div>

        {/* Validation Helper Text */}
        {currentStep === 1 && selectedChannels.length === 0 && (
          <div className="mt-4 text-sm text-red-600 dark:text-red-400">
            Please select at least one communication channel
          </div>
        )}
      </div>
    </div>
  )
}

/**
 * Get display title for current step
 */
function getStepTitle(step: number): string {
  const titles: Record<number, string> = {
    1: 'Connect Channels',
    2: 'Select AI Model',
    3: 'Choose Tools',
    4: 'Review & Launch',
  }
  return titles[step] || ''
}

export default OnboardingWizard
