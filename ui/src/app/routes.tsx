import React, { useEffect } from 'react'
import { Routes, Route, Navigate, Outlet } from 'react-router-dom'
import { QueryClientProvider } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/auth'
import { queryClient } from './query'
import Login from '@/pages/Login'
import Repositories from '@/pages/Repositories'
import RepositoryBrowser from '@/pages/RepositoryBrowser'
import UploadWizard from '@/pages/UploadWizard'
import GlobalSearch from '@/pages/GlobalSearch'
import EntryDetails from '@/pages/EntryDetails'
import { Header } from '@/components/layout/Header'
import { Toaster } from '@/components/ui/toasts'
import { JobsDock } from '@/components/ui/JobsDock'

const PrivateRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
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
          <div className="loading-spinner h-8 w-8 mx-auto mb-4"></div>
          <p>Loading...</p>
        </div>
      </div>
    )
  }

  if (!isAuthenticated) {
    return null
  }

  return <>{children}</>
}

const AppLayout: React.FC = () => {
  return (
    <div className="flex flex-col min-h-screen">
      <Header />
      <main className="main-content">
        <Outlet />
      </main>
      <Toaster toasts={[]} onRemove={() => {}} />
      <JobsDock />
    </div>
  )
}

export const AppRoutes: React.FC = () => {
  const { isAuthenticated } = useAuthStore()

  return (
    <QueryClientProvider client={queryClient}>
      <div className="min-h-screen bg-background">
        <Routes>
          <Route
            path="/login"
            element={
              isAuthenticated ? (
                <Navigate to="/repos" replace />
              ) : (
                <Login />
              )
            }
          />
          <Route
            path="/*"
            element={
              <PrivateRoute>
                <AppLayout />
              </PrivateRoute>
            }
          >
            <Route path="" element={<Navigate to="/repos" replace />} />
            <Route path="repos" element={<Repositories />} />
            <Route path="repos/:name" element={<RepositoryBrowser />} />
            <Route path="repos/:name/upload" element={<UploadWizard />} />
            <Route path="repos/:name/entry/:ref/*" element={<EntryDetails />} />
            <Route path="search" element={<GlobalSearch />} />
          </Route>
        </Routes>
      </div>
    </QueryClientProvider>
  )
}