// Mobile-optimized search help component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useCallback } from 'react'
import { 
  HelpCircle, 
  Search, 
  Filter, 
  SortAsc, 
  Tag, 
  File,
  Folder,
  ChevronRight,
  ChevronDown,
  Lightbulb,
  BookOpen,
  Video,
  MessageCircle
} from 'lucide-react'

interface HelpSection {
  id: string
  title: string
  icon: React.ComponentType<{ className?: string }>
  content: React.ReactNode
}

interface MobileSearchHelpProps {
  className?: string
}

export const MobileSearchHelp: React.FC<MobileSearchHelpProps> = ({ className = '' }) => {
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set())
  const [activeTab, setActiveTab] = useState<'basics' | 'advanced' | 'tips' | 'faq'>('basics')

  // Toggle section expansion
  const toggleSection = useCallback((sectionId: string) => {
    setExpandedSections(prev => {
      const newSet = new Set(prev)
      if (newSet.has(sectionId)) {
        newSet.delete(sectionId)
      } else {
        newSet.add(sectionId)
      }
      return newSet
    })
  }, [])

  // Get section icon
  const getSectionIcon = (sectionId: string) => {
    const isExpanded = expandedSections.has(sectionId)
    return isExpanded ? (
      <ChevronDown className="w-4 h-4 text-gray-400" />
    ) : (
      <ChevronRight className="w-4 h-4 text-gray-400" />
    )
  }

  // Basic search help
  const basicSections: HelpSection[] = [
    {
      id: 'getting-started',
      title: 'Getting Started',
      icon: Search,
      content: (
        <div className="space-y-3">
          <p className="text-sm text-gray-600">
            Start by typing your search query in the search bar. You can search for:
          </p>
          <ul className="space-y-2 text-sm text-gray-600">
            <li className="flex items-start space-x-2">
              <File className="w-4 h-4 text-blue-500 flex-shrink-0 mt-0.5" />
              <span>File names and content</span>
            </li>
            <li className="flex items-start space-x-2">
              <Folder className="w-4 h-4 text-green-500 flex-shrink-0 mt-0.5" />
              <span>Folder names and structure</span>
            </li>
            <li className="flex items-start space-x-2">
              <Tag className="w-4 h-4 text-purple-500 flex-shrink-0 mt-0.5" />
              <span>Tags and metadata</span>
            </li>
          </ul>
        </div>
      )
    },
    {
      id: 'search-filters',
      title: 'Using Filters',
      icon: Filter,
      content: (
        <div className="space-y-3">
          <p className="text-sm text-gray-600">
            Use filters to narrow down your search results:
          </p>
          <div className="space-y-2">
            <div className="p-3 bg-gray-50 rounded-lg">
              <p className="text-sm font-medium text-gray-900">File Type</p>
              <p className="text-xs text-gray-600">Filter by CSV, PDF, images, etc.</p>
            </div>
            <div className="p-3 bg-gray-50 rounded-lg">
              <p className="text-sm font-medium text-gray-900">Date Range</p>
              <p className="text-xs text-gray-600">Find files from specific time periods</p>
            </div>
            <div className="p-3 bg-gray-50 rounded-lg">
              <p className="text-sm font-medium text-gray-900">Repository</p>
              <p className="text-xs text-gray-600">Search within specific repositories</p>
            </div>
          </div>
        </div>
      )
    },
    {
      id: 'search-sort',
      title: 'Sorting Results',
      icon: SortAsc,
      content: (
        <div className="space-y-3">
          <p className="text-sm text-gray-600">
            Sort your search results by different criteria:
          </p>
          <div className="space-y-2">
            <div className="flex items-center justify-between p-2 bg-gray-50 rounded">
              <span className="text-sm text-gray-700">Relevance</span>
              <span className="text-xs text-gray-500">Best matches first</span>
            </div>
            <div className="flex items-center justify-between p-2 bg-gray-50 rounded">
              <span className="text-sm text-gray-700">Date Modified</span>
              <span className="text-xs text-gray-500">Newest first</span>
            </div>
            <div className="flex items-center justify-between p-2 bg-gray-50 rounded">
              <span className="text-sm text-gray-700">File Size</span>
              <span className="text-xs text-gray-500">Largest first</span>
            </div>
            <div className="flex items-center justify-between p-2 bg-gray-50 rounded">
              <span className="text-sm text-gray-700">Name</span>
              <span className="text-xs text-gray-500">Alphabetical</span>
            </div>
          </div>
        </div>
      )
    }
  ]

  // Advanced search help
  const advancedSections: HelpSection[] = [
    {
      id: 'search-operators',
      title: 'Search Operators',
      icon: Search,
      content: (
        <div className="space-y-3">
          <p className="text-sm text-gray-600">
            Use these operators to refine your search:
          </p>
          <div className="space-y-2">
            <div className="p-3 bg-gray-50 rounded-lg">
              <p className="text-sm font-medium text-gray-900">"exact phrase"</p>
              <p className="text-xs text-gray-600">Search for exact phrases</p>
            </div>
            <div className="p-3 bg-gray-50 rounded-lg">
              <p className="text-sm font-medium text-gray-900">AND, OR, NOT</p>
              <p className="text-xs text-gray-600">Combine search terms logically</p>
            </div>
            <div className="p-3 bg-gray-50 rounded-lg">
              <p className="text-sm font-medium text-gray-900">*wildcard*</p>
              <p className="text-xs text-gray-600">Use asterisks for partial matches</p>
            </div>
            <div className="p-3 bg-gray-50 rounded-lg">
              <p className="text-sm font-medium text-gray-900">filetype:csv</p>
              <p className="text-xs text-gray-600">Search by file type</p>
            </div>
          </div>
        </div>
      )
    },
    {
      id: 'semantic-search',
      title: 'Semantic Search',
      icon: Lightbulb,
      content: (
        <div className="space-y-3">
          <p className="text-sm text-gray-600">
            Semantic search understands the meaning behind your queries:
          </p>
          <div className="space-y-2">
            <div className="p-3 bg-blue-50 rounded-lg">
              <p className="text-sm font-medium text-blue-900">Natural Language</p>
              <p className="text-xs text-blue-700">Ask questions in plain English</p>
            </div>
            <div className="p-3 bg-blue-50 rounded-lg">
              <p className="text-sm font-medium text-blue-900">Context Understanding</p>
              <p className="text-xs text-blue-700">Finds related content automatically</p>
            </div>
            <div className="p-3 bg-blue-50 rounded-lg">
              <p className="text-sm font-medium text-blue-900">Smart Suggestions</p>
              <p className="text-xs text-blue-700">Get AI-powered search suggestions</p>
            </div>
          </div>
        </div>
      )
    }
  ]

  // Search tips
  const searchTips = [
    {
      icon: Lightbulb,
      title: 'Use specific terms',
      description: 'Be specific with your search terms for better results'
    },
    {
      icon: Tag,
      title: 'Try different keywords',
      description: 'If you don\'t find what you\'re looking for, try synonyms'
    },
    {
      icon: Filter,
      title: 'Use filters',
      description: 'Narrow down results with file type, date, or repository filters'
    },
    {
      icon: Search,
      title: 'Check spelling',
      description: 'Make sure your search terms are spelled correctly'
    },
    {
      icon: File,
      title: 'Search file content',
      description: 'Search within file contents, not just file names'
    },
    {
      icon: Folder,
      title: 'Browse by folder',
      description: 'Use the repository browser to explore by folder structure'
    }
  ]

  // FAQ
  const faqItems = [
    {
      question: 'How do I search for files with specific extensions?',
      answer: 'Use the filetype: operator followed by the extension. For example, "filetype:csv" to find CSV files.'
    },
    {
      question: 'Can I search within specific repositories?',
      answer: 'Yes, use the repository filter or include the repository name in your search query.'
    },
    {
      question: 'What is semantic search?',
      answer: 'Semantic search uses AI to understand the meaning of your query and find relevant content even if it doesn\'t contain your exact search terms.'
    },
    {
      question: 'How do I save my search results?',
      answer: 'You can bookmark search results or export them to CSV/JSON format using the export options.'
    },
    {
      question: 'Why am I not getting results?',
      answer: 'Try broadening your search terms, checking your spelling, or using different keywords. You can also try semantic search for better results.'
    }
  ]

  return (
    <div className={`bg-white ${className}`}>
      {/* Header */}
      <div className="p-4 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold text-gray-900">
              Search Help
            </h2>
            <p className="text-sm text-gray-600">
              Learn how to get the most out of search
            </p>
          </div>
          
          <HelpCircle className="w-6 h-6 text-gray-400" />
        </div>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-gray-200">
        {[
          { id: 'basics', label: 'Basics', icon: BookOpen },
          { id: 'advanced', label: 'Advanced', icon: Search },
          { id: 'tips', label: 'Tips', icon: Lightbulb },
          { id: 'faq', label: 'FAQ', icon: MessageCircle },
        ].map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => setActiveTab(id as any)}
            className={`flex-1 flex items-center justify-center space-x-2 py-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab === id
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            }`}
          >
            <Icon className="w-4 h-4" />
            <span>{label}</span>
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="p-4">
        {activeTab === 'basics' && (
          <div className="space-y-4">
            {basicSections.map((section) => {
              const Icon = section.icon
              const isExpanded = expandedSections.has(section.id)
              
              return (
                <div key={section.id} className="border border-gray-200 rounded-lg">
                  <button
                    onClick={() => toggleSection(section.id)}
                    className="w-full flex items-center justify-between p-4 text-left hover:bg-gray-50 transition-colors"
                  >
                    <div className="flex items-center space-x-3">
                      <Icon className="w-5 h-5 text-gray-600" />
                      <span className="text-sm font-medium text-gray-900">
                        {section.title}
                      </span>
                    </div>
                    {getSectionIcon(section.id)}
                  </button>
                  
                  {isExpanded && (
                    <div className="px-4 pb-4 border-t border-gray-100">
                      {section.content}
                    </div>
                  )}
                </div>
              )
            })}
          </div>
        )}

        {activeTab === 'advanced' && (
          <div className="space-y-4">
            {advancedSections.map((section) => {
              const Icon = section.icon
              const isExpanded = expandedSections.has(section.id)
              
              return (
                <div key={section.id} className="border border-gray-200 rounded-lg">
                  <button
                    onClick={() => toggleSection(section.id)}
                    className="w-full flex items-center justify-between p-4 text-left hover:bg-gray-50 transition-colors"
                  >
                    <div className="flex items-center space-x-3">
                      <Icon className="w-5 h-5 text-gray-600" />
                      <span className="text-sm font-medium text-gray-900">
                        {section.title}
                      </span>
                    </div>
                    {getSectionIcon(section.id)}
                  </button>
                  
                  {isExpanded && (
                    <div className="px-4 pb-4 border-t border-gray-100">
                      {section.content}
                    </div>
                  )}
                </div>
              )
            })}
          </div>
        )}

        {activeTab === 'tips' && (
          <div className="space-y-4">
            <h3 className="text-sm font-medium text-gray-900">Search Tips</h3>
            <div className="grid grid-cols-1 gap-3">
              {searchTips.map((tip, index) => {
                const Icon = tip.icon
                return (
                  <div key={index} className="flex items-start space-x-3 p-3 bg-gray-50 rounded-lg">
                    <Icon className="w-5 h-5 text-blue-600 flex-shrink-0 mt-0.5" />
                    <div>
                      <p className="text-sm font-medium text-gray-900">{tip.title}</p>
                      <p className="text-xs text-gray-600 mt-1">{tip.description}</p>
                    </div>
                  </div>
                )
              })}
            </div>
          </div>
        )}

        {activeTab === 'faq' && (
          <div className="space-y-4">
            <h3 className="text-sm font-medium text-gray-900">Frequently Asked Questions</h3>
            <div className="space-y-3">
              {faqItems.map((item, index) => (
                <div key={index} className="border border-gray-200 rounded-lg">
                  <button
                    onClick={() => toggleSection(`faq-${index}`)}
                    className="w-full flex items-center justify-between p-4 text-left hover:bg-gray-50 transition-colors"
                  >
                    <span className="text-sm font-medium text-gray-900">
                      {item.question}
                    </span>
                    {getSectionIcon(`faq-${index}`)}
                  </button>
                  
                  {expandedSections.has(`faq-${index}`) && (
                    <div className="px-4 pb-4 border-t border-gray-100">
                      <p className="text-sm text-gray-600">{item.answer}</p>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default MobileSearchHelp
