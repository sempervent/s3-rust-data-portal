import React from 'react'
import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import { UserManager, User } from 'oidc-client-ts'

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
  
  // Actions
  signinRedirect: () => Promise<void>
  signoutRedirect: () => Promise<void>
  handleCallback: () => Promise<void>
  initialize: () => Promise<void>
}

export const useAuthStore = create<AuthState>()(
  devtools(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      isLoading: true,

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
