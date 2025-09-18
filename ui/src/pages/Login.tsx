import React, { useEffect } from 'react'
import { useAuthStore } from '@/stores/auth'
import { Button } from '@/components/ui/button'

const Login: React.FC = () => {
  const { isAuthenticated, isLoading, signinRedirect } = useAuthStore()

  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      signinRedirect()
    }
  }, [isAuthenticated, isLoading, signinRedirect])

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
          <p>Loading...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-background">
      <div className="max-w-md w-full space-y-8 p-8">
        <div className="text-center">
          <h2 className="text-3xl font-bold">Welcome to Blacklake</h2>
          <p className="mt-2 text-muted-foreground">
            Sign in to access your data repositories
          </p>
        </div>
        
        <div className="space-y-4">
          <Button
            onClick={() => signinRedirect()}
            className="w-full"
            size="lg"
          >
            Sign In
          </Button>
        </div>
      </div>
    </div>
  )
}

export default Login