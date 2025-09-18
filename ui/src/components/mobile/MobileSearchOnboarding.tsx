// Mobile-optimized search onboarding component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useCallback } from 'react'
import { 
  Search, 
  Filter, 
  SortAsc, 
  Tag, 
  File,
  Folder,
  ChevronRight,
  ChevronLeft,
  X,
  CheckCircle,
  Lightbulb,
  Zap
} from 'lucide-react'

interface OnboardingStep {
  id: string
  title: string
  description: string
  icon: React.ComponentType<{ className?: string }>
  content: React.ReactNode
  action?: {
    label: string
    onClick: () => void
  }
}

interface MobileSearchOnboardingProps {
  isOpen: boolean
  onClose: () => void
  onComplete: () => void
  className?: string
}

export const MobileSearchOnboarding: React.FC<MobileSearchOnboardingProps> = ({
  isOpen,
  onClose,
  onComplete,
  className = ''
}) => {
  const [currentStep, setCurrentStep] = useState(0)
  const [completedSteps, setCompletedSteps] = useState<Set<number>>(new Set())

  // Onboarding steps
  const steps: OnboardingStep[] = [
    {
      id: 'welcome',
      title: 'Welcome to Search',
      description: 'Learn how to find what you need quickly and efficiently',
      icon: Search,
      content: (
        <div className="space-y-4">
          <div className="text-center">
            <div className="w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <Search className="w-8 h-8 text-blue-600" />
            </div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">
              Powerful Search at Your Fingertips
            </h3>
            <p className="text-sm text-gray-600">
              Search across all your repositories, files, and data with advanced filtering and AI-powered suggestions.
            </p>
          </div>
          
          <div className="space-y-3">
            <div className="flex items-center space-x-3">
              <CheckCircle className="w-5 h-5 text-green-500" />
              <span className="text-sm text-gray-700">Keyword and semantic search</span>
            </div>
            <div className="flex items-center space-x-3">
              <CheckCircle className="w-5 h-5 text-green-500" />
              <span className="text-sm text-gray-700">Advanced filtering options</span>
            </div>
            <div className="flex items-center space-x-3">
              <CheckCircle className="w-5 h-5 text-green-500" />
              <span className="text-sm text-gray-700">Smart search suggestions</span>
            </div>
            <div className="flex items-center space-x-3">
              <CheckCircle className="w-5 h-5 text-green-500" />
              <span className="text-sm text-gray-700">Mobile-optimized interface</span>
            </div>
          </div>
        </div>
      )
    },
    {
      id: 'basic-search',
      title: 'Basic Search',
      description: 'Start with simple text searches',
      icon: File,
      content: (
        <div className="space-y-4">
          <div className="text-center">
            <div className="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <File className="w-8 h-8 text-green-600" />
            </div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">
              Simple Text Search
            </h3>
            <p className="text-sm text-gray-600">
              Type any word or phrase to search across file names, content, and metadata.
            </p>
          </div>
          
          <div className="space-y-3">
            <div className="p-3 bg-gray-50 rounded-lg">
              <p className="text-sm font-medium text-gray-900">Try searching for:</p>
              <div className="mt-2 space-y-1">
                <p className="text-xs text-gray-600">• "sales data"</p>
                <p className="text-xs text-gray-600">• "customer analytics"</p>
                <p className="text-xs text-gray-600">• "Q4 report"</p>
              </div>
            </div>
            
            <div className="p-3 bg-blue-50 rounded-lg">
              <div className="flex items-start space-x-2">
                <Lightbulb className="w-4 h-4 text-blue-600 flex-shrink-0 mt-0.5" />
                <div>
                  <p className="text-sm font-medium text-blue-900">Pro Tip</p>
                  <p className="text-xs text-blue-700">
                    Use specific terms for better results. "Q4 sales report" is better than just "report".
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      )
    },
    {
      id: 'filters',
      title: 'Using Filters',
      description: 'Narrow down your search results',
      icon: Filter,
      content: (
        <div className="space-y-4">
          <div className="text-center">
            <div className="w-16 h-16 bg-purple-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <Filter className="w-8 h-8 text-purple-600" />
            </div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">
              Smart Filtering
            </h3>
            <p className="text-sm text-gray-600">
              Use filters to find exactly what you're looking for.
            </p>
          </div>
          
          <div className="space-y-3">
            <div className="grid grid-cols-2 gap-2">
              <div className="p-3 bg-gray-50 rounded-lg text-center">
                <File className="w-6 h-6 text-blue-500 mx-auto mb-2" />
                <p className="text-xs font-medium text-gray-900">File Type</p>
                <p className="text-xs text-gray-600">CSV, PDF, etc.</p>
              </div>
              <div className="p-3 bg-gray-50 rounded-lg text-center">
                <Folder className="w-6 h-6 text-green-500 mx-auto mb-2" />
                <p className="text-xs font-medium text-gray-900">Repository</p>
                <p className="text-xs text-gray-600">Specific repos</p>
              </div>
              <div className="p-3 bg-gray-50 rounded-lg text-center">
                <Tag className="w-6 h-6 text-purple-500 mx-auto mb-2" />
                <p className="text-xs font-medium text-gray-900">Tags</p>
                <p className="text-xs text-gray-600">Metadata tags</p>
              </div>
              <div className="p-3 bg-gray-50 rounded-lg text-center">
                <SortAsc className="w-6 h-6 text-orange-500 mx-auto mb-2" />
                <p className="text-xs font-medium text-gray-900">Date</p>
                <p className="text-xs text-gray-600">Time range</p>
              </div>
            </div>
          </div>
        </div>
      )
    },
    {
      id: 'semantic-search',
      title: 'Semantic Search',
      description: 'AI-powered search understanding',
      icon: Zap,
      content: (
        <div className="space-y-4">
          <div className="text-center">
            <div className="w-16 h-16 bg-yellow-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <Zap className="w-8 h-8 text-yellow-600" />
            </div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">
              AI-Powered Search
            </h3>
            <p className="text-sm text-gray-600">
              Ask questions in natural language and get intelligent results.
            </p>
          </div>
          
          <div className="space-y-3">
            <div className="p-3 bg-yellow-50 rounded-lg">
              <p className="text-sm font-medium text-yellow-900">Try asking:</p>
              <div className="mt-2 space-y-1">
                <p className="text-xs text-yellow-700">• "Show me customer data from last month"</p>
                <p className="text-xs text-yellow-700">• "Find files related to marketing campaigns"</p>
                <p className="text-xs text-yellow-700">• "What datasets contain sales information?"</p>
              </div>
            </div>
            
            <div className="p-3 bg-blue-50 rounded-lg">
              <div className="flex items-start space-x-2">
                <Lightbulb className="w-4 h-4 text-blue-600 flex-shrink-0 mt-0.5" />
                <div>
                  <p className="text-sm font-medium text-blue-900">How it works</p>
                  <p className="text-xs text-blue-700">
                    Semantic search understands the meaning behind your queries and finds relevant content even if it doesn't contain your exact words.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      )
    },
    {
      id: 'complete',
      title: 'You\'re All Set!',
      description: 'Start exploring your data',
      icon: CheckCircle,
      content: (
        <div className="space-y-4">
          <div className="text-center">
            <div className="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <CheckCircle className="w-8 h-8 text-green-600" />
            </div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">
              Ready to Search!
            </h3>
            <p className="text-sm text-gray-600">
              You now know the basics of searching. Start exploring your data!
            </p>
          </div>
          
          <div className="space-y-3">
            <div className="p-3 bg-green-50 rounded-lg">
              <p className="text-sm font-medium text-green-900">What's next?</p>
              <div className="mt-2 space-y-1">
                <p className="text-xs text-green-700">• Try a search query</p>
                <p className="text-xs text-green-700">• Explore different filters</p>
                <p className="text-xs text-green-700">• Test semantic search</p>
                <p className="text-xs text-green-700">• Bookmark useful results</p>
              </div>
            </div>
            
            <div className="p-3 bg-blue-50 rounded-lg">
              <div className="flex items-start space-x-2">
                <Lightbulb className="w-4 h-4 text-blue-600 flex-shrink-0 mt-0.5" />
                <div>
                  <p className="text-sm font-medium text-blue-900">Need help?</p>
                  <p className="text-xs text-blue-700">
                    Check out the help section for more tips and advanced features.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      )
    }
  ]

  // Handle next step
  const handleNext = useCallback(() => {
    if (currentStep < steps.length - 1) {
      setCompletedSteps(prev => new Set([...prev, currentStep]))
      setCurrentStep(prev => prev + 1)
    } else {
      onComplete()
    }
  }, [currentStep, steps.length, onComplete])

  // Handle previous step
  const handlePrevious = useCallback(() => {
    if (currentStep > 0) {
      setCurrentStep(prev => prev - 1)
    }
  }, [currentStep])

  // Handle step completion
  const handleStepComplete = useCallback((stepIndex: number) => {
    setCompletedSteps(prev => new Set([...prev, stepIndex]))
  }, [])

  if (!isOpen) return null

  const currentStepData = steps[currentStep]
  const Icon = currentStepData.icon
  const isLastStep = currentStep === steps.length - 1

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4">
      <div className="w-full max-w-md bg-white rounded-lg shadow-xl">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200">
          <div className="flex items-center space-x-3">
            <Icon className="w-6 h-6 text-blue-600" />
            <div>
              <h2 className="text-lg font-semibold text-gray-900">
                {currentStepData.title}
              </h2>
              <p className="text-sm text-gray-600">
                {currentStepData.description}
              </p>
            </div>
          </div>
          
          <button
            onClick={onClose}
            className="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-md transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Progress */}
        <div className="px-4 py-3 border-b border-gray-200">
          <div className="flex items-center justify-between text-xs text-gray-500 mb-2">
            <span>Step {currentStep + 1} of {steps.length}</span>
            <span>{Math.round(((currentStep + 1) / steps.length) * 100)}%</span>
          </div>
          <div className="w-full bg-gray-200 rounded-full h-2">
            <div
              className="bg-blue-600 h-2 rounded-full transition-all duration-300"
              style={{ width: `${((currentStep + 1) / steps.length) * 100}%` }}
            />
          </div>
        </div>

        {/* Content */}
        <div className="p-4">
          {currentStepData.content}
        </div>

        {/* Actions */}
        <div className="flex items-center justify-between p-4 border-t border-gray-200">
          <button
            onClick={handlePrevious}
            disabled={currentStep === 0}
            className="flex items-center space-x-2 px-4 py-2 text-gray-600 hover:text-gray-900 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            <ChevronLeft className="w-4 h-4" />
            <span className="text-sm">Previous</span>
          </button>
          
          <div className="flex items-center space-x-2">
            {steps.map((_, index) => (
              <div
                key={index}
                className={`w-2 h-2 rounded-full transition-colors ${
                  index === currentStep
                    ? 'bg-blue-600'
                    : index < currentStep
                    ? 'bg-green-500'
                    : 'bg-gray-300'
                }`}
              />
            ))}
          </div>
          
          <button
            onClick={handleNext}
            className="flex items-center space-x-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
          >
            <span className="text-sm">
              {isLastStep ? 'Get Started' : 'Next'}
            </span>
            {!isLastStep && <ChevronRight className="w-4 h-4" />}
          </button>
        </div>
      </div>
    </div>
  )
}

export default MobileSearchOnboarding
