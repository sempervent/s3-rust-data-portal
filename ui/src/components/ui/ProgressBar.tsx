import React from 'react'

interface ProgressBarProps {
  value: number // 0-100
  max?: number
  className?: string
  showText?: boolean
  text?: string
  variant?: 'default' | 'success' | 'error' | 'warning'
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  value,
  max = 100,
  className = "",
  showText = true,
  text,
  variant = 'default'
}) => {
  const percentage = Math.min(100, Math.max(0, (value / max) * 100))
  
  const variantClasses = {
    default: 'bg-primary',
    success: 'bg-green-500',
    error: 'bg-destructive',
    warning: 'bg-yellow-500'
  }

  return (
    <div className={`w-full ${className}`}>
      <div className="w-full bg-secondary rounded-full h-2.5 overflow-hidden">
        <div
          className={`h-2.5 transition-all duration-300 ease-out ${variantClasses[variant]}`}
          style={{ width: `${percentage}%` }}
        />
      </div>
      {showText && (
        <div className="text-sm text-muted-foreground mt-1 text-center">
          {text || `${Math.round(percentage)}%`}
        </div>
      )}
    </div>
  )
}
