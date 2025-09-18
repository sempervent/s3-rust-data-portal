import React from 'react'
import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import { UserManager, User } from 'oidc-client-ts'
import { api } from '@/lib/api'

const userManager = new UserManager({
  authority: import.meta.env.VITE_OIDC_ISSUER || 'http://localhost:8081/realms/master',
  client_id: import.meta.env.VITE_OIDC_CLIENT_ID || 'blacklake',
  redirect_uri: import.meta.env.VITE_OIDC_REDIRECT_URI || 'http://localhost:5173/callback',
  response_type: 'code',
  scope: import.meta.env.VITE_OIDC_SCOPE || 'openid profile email',
  automaticSilentRenew: true,
  silent_redirect_uri: `${window.location.origin}/silent-callback.html`,
})

interface AuthUser {
  sub: string
  name?: string
  email?: string
  groups?: string[]
  profile: {
    sub: string
    name?: string
    email?: string
    groups?: string[]
  }
  access_token: string
}

interface AuthState {
  user: AuthUser | null
  isAuthenticated: boolean
  isLoading: boolean
  csrfToken: string | null
  authMethod: 'oidc' | 'session'
  
  // Actions
  signinRedirect: () => Promise<void>
  signoutRedirect: () => Promise<void>
  handleCallback: () => Promise<void>
  initialize: () => Promise<void>
  createSession: (oidcToken: string) => Promise<void>
  getCsrfToken: () => Promise<string>
  logout: () => Promise<void>
}

export const useAuthStore = create<AuthState>()(
  devtools(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      isLoading: true,
      csrfToken: null,
      authMethod: 'session', // Default to session-based auth for web UI

      signinRedirect: async () => {
        try {
          await userManager.signinRedirect()
        } catch (error) {
          console.error('Login failed:', error)
        }
      },

      signoutRedirect: async () => {
        try {
          await userManager.signoutRedirect()
          set({ user: null, isAuthenticated: false })
        } catch (error) {
          console.error('Logout failed:', error)
        }
      },

      handleCallback: async () => {
        try {
          const oidcUser = await userManager.signinRedirectCallback()
          const user: AuthUser = {
            sub: oidcUser.profile.sub!,
            name: oidcUser.profile.name,
            email: oidcUser.profile.email,
            groups: oidcUser.profile.groups as string[],
            profile: {
              sub: oidcUser.profile.sub!,
              name: oidcUser.profile.name,
              email: oidcUser.profile.email,
              groups: oidcUser.profile.groups as string[],
            },
            access_token: oidcUser.access_token,
          }
          
          // Create session with OIDC token
          await get().createSession(oidcUser.access_token)
          
          set({
            user,
            isAuthenticated: true,
            isLoading: false,
          })
        } catch (error) {
          console.error('Callback handling failed:', error)
          set({ isLoading: false })
        }
      },

      initialize: async () => {
        try {
          const oidcUser = await userManager.getUser()
          if (oidcUser && !oidcUser.expired) {
            const user: AuthUser = {
              sub: oidcUser.profile.sub!,
              name: oidcUser.profile.name,
              email: oidcUser.profile.email,
              groups: oidcUser.profile.groups as string[],
              profile: {
                sub: oidcUser.profile.sub!,
                name: oidcUser.profile.name,
                email: oidcUser.profile.email,
                groups: oidcUser.profile.groups as string[],
              },
              access_token: oidcUser.access_token,
            }
            
            set({
              user,
              isAuthenticated: true,
              isLoading: false,
            })
          } else {
            set({ isLoading: false })
          }
        } catch (error) {
          console.error('Auth initialization failed:', error)
          set({ isLoading: false })
        }
      },

      createSession: async (oidcToken: string) => {
        try {
          const response = await api.post('/v1/session/login', {
            oidc_token: oidcToken
          })
          
          if (response.data.success) {
            // Get CSRF token for future requests
            await get().getCsrfToken()
          }
        } catch (error) {
          console.error('Session creation failed:', error)
          throw error
        }
      },

      getCsrfToken: async () => {
        try {
          const response = await api.get('/v1/csrf')
          const csrfToken = response.data.data.csrf_token
          set({ csrfToken })
          return csrfToken
        } catch (error) {
          console.error('CSRF token retrieval failed:', error)
          throw error
        }
      },

      logout: async () => {
        try {
          // Get CSRF token if not available
          let csrfToken = get().csrfToken
          if (!csrfToken) {
            csrfToken = await get().getCsrfToken()
          }

          // Logout from session
          await api.post('/v1/session/logout', {}, {
            headers: {
              'x-csrf-token': csrfToken
            }
          })

          // Clear local state
          set({
            user: null,
            isAuthenticated: false,
            csrfToken: null
          })

          // Also logout from OIDC
          await get().signoutRedirect()
        } catch (error) {
          console.error('Logout failed:', error)
          // Clear local state even if server logout fails
          set({
            user: null,
            isAuthenticated: false,
            csrfToken: null
          })
        }
      },
    }),
    {
      name: 'auth-store',
    }
  )
)

// Auth Provider component
export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { initialize } = useAuthStore()

  React.useEffect(() => {
    initialize()
  }, [initialize])

  return <>{children}</>
}
