import React, { useState } from 'react';
import { HelpCircle, X, ChevronDown, ChevronUp } from 'lucide-react';

interface ContextualHelpProps {
  topic: string;
  section?: string;
  title?: string;
  children?: React.ReactNode;
  className?: string;
  position?: 'top' | 'bottom' | 'left' | 'right';
}

const ContextualHelp: React.FC<ContextualHelpProps> = ({
  topic,
  section,
  title,
  children,
  className = '',
  position = 'bottom'
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);

  const getHelpUrl = () => {
    const baseUrl = process.env.REACT_APP_DOCS_URL || 'https://docs.blacklake.dev';
    let url = baseUrl;
    
    if (topic) {
      url += `/${topic}`;
    }
    
    if (section) {
      url += `#${section}`;
    }
    
    return url;
  };

  const handleOpenDocs = () => {
    const url = getHelpUrl();
    window.open(url, '_blank', 'noopener,noreferrer');
  };

  const positionClasses = {
    top: 'bottom-full mb-2',
    bottom: 'top-full mt-2',
    left: 'right-full mr-2',
    right: 'left-full ml-2'
  };

  const helpContent = {
    search: {
      title: 'Search Help',
      content: 'Learn how to use BlackLake\'s powerful search features including faceted search, semantic search, and saved searches.',
      sections: [
        { title: 'Basic Search', anchor: 'basic-search' },
        { title: 'Advanced Search', anchor: 'advanced-search' },
        { title: 'Faceted Search', anchor: 'faceted-search' },
        { title: 'Semantic Search', anchor: 'semantic-search' }
      ]
    },
    upload: {
      title: 'Upload Help',
      content: 'Learn how to upload files to BlackLake, including supported formats, metadata extraction, and version control.',
      sections: [
        { title: 'Supported Formats', anchor: 'supported-formats' },
        { title: 'Metadata Extraction', anchor: 'metadata-extraction' },
        { title: 'Version Control', anchor: 'version-control' },
        { title: 'File Organization', anchor: 'file-organization' }
      ]
    },
    admin: {
      title: 'Admin Help',
      content: 'Learn how to manage users, permissions, repositories, and system settings in BlackLake.',
      sections: [
        { title: 'User Management', anchor: 'user-management' },
        { title: 'Permission Management', anchor: 'permission-management' },
        { title: 'Repository Management', anchor: 'repository-management' },
        { title: 'System Settings', anchor: 'system-settings' }
      ]
    },
    compliance: {
      title: 'Compliance Help',
      content: 'Learn about BlackLake\'s compliance features including retention policies, legal holds, and audit logging.',
      sections: [
        { title: 'Retention Policies', anchor: 'retention-policies' },
        { title: 'Legal Holds', anchor: 'legal-holds' },
        { title: 'Audit Logging', anchor: 'audit-logging' },
        { title: 'Access Reviews', anchor: 'access-reviews' }
      ]
    }
  };

  const helpData = helpContent[topic as keyof typeof helpContent] || {
    title: title || 'Help',
    content: 'Get help with this feature.',
    sections: []
  };

  return (
    <div className={`relative inline-block ${className}`}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="text-gray-400 hover:text-gray-600 transition-colors"
        title="Get help"
      >
        <HelpCircle className="w-4 h-4" />
      </button>

      {isOpen && (
        <div className={`absolute z-50 w-80 bg-white rounded-lg shadow-lg border border-gray-200 ${positionClasses[position]}`}>
          <div className="p-4">
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-lg font-semibold text-gray-900">
                {helpData.title}
              </h3>
              <button
                onClick={() => setIsOpen(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <p className="text-sm text-gray-600 mb-4">
              {helpData.content}
            </p>

            {helpData.sections.length > 0 && (
              <div className="mb-4">
                <button
                  onClick={() => setIsExpanded(!isExpanded)}
                  className="flex items-center text-sm text-blue-600 hover:text-blue-800"
                >
                  {isExpanded ? <ChevronUp className="w-4 h-4 mr-1" /> : <ChevronDown className="w-4 h-4 mr-1" />}
                  Quick Links
                </button>

                {isExpanded && (
                  <div className="mt-2 space-y-1">
                    {helpData.sections.map((section, index) => (
                      <a
                        key={index}
                        href={`${getHelpUrl()}#${section.anchor}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="block text-sm text-gray-600 hover:text-gray-800 py-1"
                      >
                        • {section.title}
                      </a>
                    ))}
                  </div>
                )}
              </div>
            )}

            {children && (
              <div className="mb-4">
                {children}
              </div>
            )}

            <div className="flex space-x-2">
              <button
                onClick={handleOpenDocs}
                className="flex-1 bg-blue-600 text-white px-3 py-2 rounded text-sm hover:bg-blue-700 transition-colors"
              >
                Open Documentation
              </button>
              <button
                onClick={() => setIsOpen(false)}
                className="px-3 py-2 text-sm text-gray-600 hover:text-gray-800 transition-colors"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default ContextualHelp;
