import React from 'react';
import { HelpCircle, ExternalLink } from 'lucide-react';

interface HelpIconProps {
  topic?: string;
  section?: string;
  className?: string;
  size?: 'sm' | 'md' | 'lg';
  variant?: 'icon' | 'button' | 'link';
}

const HelpIcon: React.FC<HelpIconProps> = ({
  topic,
  section,
  className = '',
  size = 'md',
  variant = 'icon'
}) => {
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

  const handleClick = () => {
    const url = getHelpUrl();
    window.open(url, '_blank', 'noopener,noreferrer');
  };

  const sizeClasses = {
    sm: 'w-4 h-4',
    md: 'w-5 h-5',
    lg: 'w-6 h-6'
  };

  const variantClasses = {
    icon: 'text-gray-400 hover:text-gray-600 cursor-pointer transition-colors',
    button: 'inline-flex items-center px-2 py-1 text-sm text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition-colors',
    link: 'inline-flex items-center text-blue-600 hover:text-blue-800 underline transition-colors'
  };

  return (
    <div
      className={`${variantClasses[variant]} ${className}`}
      onClick={handleClick}
      title={`Get help with ${topic || 'this feature'}`}
    >
      <HelpCircle className={sizeClasses[size]} />
      {variant === 'button' && (
        <span className="ml-1">Help</span>
      )}
      {variant === 'link' && (
        <span className="ml-1">Documentation</span>
      )}
      <ExternalLink className="w-3 h-3 ml-1" />
    </div>
  );
};

export default HelpIcon;
