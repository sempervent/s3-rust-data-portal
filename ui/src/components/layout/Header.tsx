import React from 'react'
import { useAuthStore } from '@/stores/auth'
import { useAppStore } from '@/app/store'
import { Button } from '@/components/ui/button'
import { useJobSimulation } from '@/components/ui/JobsDock'

export const Header: React.FC = () => {
  const { user, signoutRedirect } = useAuthStore()
  const { currentRepo } = useAppStore()
  const { simulateJob } = useJobSimulation()

  const handleSimulateJob = () => {
    const jobTypes = ['sampling', 'rdf', 'antivirus', 'upload', 'metadata'] as const
    const randomType = jobTypes[Math.floor(Math.random() * jobTypes.length)]
    const repo = currentRepo || 'demo-repo'
    const path = `data/file-${Date.now()}.csv`
    
    simulateJob(randomType, repo, path)
  }

  return (
    <header className="border-b bg-background">
      <div className="container mx-auto px-4 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <h1 className="text-xl font-bold">Blacklake Data Portal</h1>
            {currentRepo && (
              <span className="text-sm text-muted-foreground">
                {currentRepo}
              </span>
            )}
          </div>
          
          <div className="flex items-center space-x-4">
            {/* Demo Job Simulator */}
            <Button
              variant="ghost"
              size="sm"
              onClick={handleSimulateJob}
              className="text-xs"
            >
              🎯 Demo Job
            </Button>
            
            {user && (
              <span className="text-sm">
                Welcome, {user.profile.name || user.profile.email}
              </span>
            )}
            <Button
              variant="outline"
              size="sm"
              onClick={() => signoutRedirect()}
            >
              Sign Out
            </Button>
          </div>
        </div>
      </div>
    </header>
  )
}