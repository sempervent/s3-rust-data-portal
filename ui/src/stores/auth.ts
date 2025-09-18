import { create } from 'zustand';
import { UserManager, User } from 'oidc-client-ts';
import { AuthState } from '../types';

const userManager = new UserManager({
  authority: import.meta.env.VITE_OIDC_ISSUER,
  client_id: import.meta.env.VITE_OIDC_CLIENT_ID,
  redirect_uri: import.meta.env.VITE_OIDC_REDIRECT_URI,
  response_type: 'code',
  scope: import.meta.env.VITE_OIDC_SCOPE,
  automaticSilentRenew: true,
  silent_redirect_uri: `${window.location.origin}/silent-callback.html`,
});

interface AuthStore extends AuthState {
  login: () => Promise<void>;
  logout: () => Promise<void>;
  handleCallback: () => Promise<void>;
  initialize: () => Promise<void>;
}

export const useAuthStore = create<AuthStore>((set, get) => ({
  user: null,
  token: null,
  isAuthenticated: false,
  isLoading: true,

  login: async () => {
    try {
      await userManager.signinRedirect();
    } catch (error) {
      console.error('Login failed:', error);
    }
  },

  logout: async () => {
    try {
      await userManager.signoutRedirect();
      set({ user: null, token: null, isAuthenticated: false });
    } catch (error) {
      console.error('Logout failed:', error);
    }
  },

  handleCallback: async () => {
    try {
      const user = await userManager.signinRedirectCallback();
      set({
        user: {
          sub: user.profile.sub,
          name: user.profile.name,
          email: user.profile.email,
          groups: user.profile.groups,
        },
        token: user.access_token,
        isAuthenticated: true,
        isLoading: false,
      });
    } catch (error) {
      console.error('Callback handling failed:', error);
      set({ isLoading: false });
    }
  },

  initialize: async () => {
    try {
      const user = await userManager.getUser();
      if (user && !user.expired) {
        set({
          user: {
            sub: user.profile.sub,
            name: user.profile.name,
            email: user.profile.email,
            groups: user.profile.groups,
          },
          token: user.access_token,
          isAuthenticated: true,
          isLoading: false,
        });
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      console.error('Auth initialization failed:', error);
      set({ isLoading: false });
    }
  },
}));
