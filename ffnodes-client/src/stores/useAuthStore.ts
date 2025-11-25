import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface User {
  id: string;
  email: string;
  name: string;
  avatar?: string;
  provider: 'google' | 'github' | 'microsoft' | 'facebook' | 'display_name';
}

export interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  accessToken: string | null;
  refreshToken: string | null;
  expiresAt: number | null;

  // Actions
  setUser: (user: User) => void;
  setTokens: (accessToken: string, refreshToken: string, expiresIn: number) => void;
  logout: () => void;
  isTokenExpired: () => boolean;

  // Display name auth (simple mode)
  loginWithDisplayName: (displayName: string) => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      accessToken: null,
      refreshToken: null,
      expiresAt: null,

      setUser: (user) => {
        set({ user, isAuthenticated: true });
      },

      setTokens: (accessToken, refreshToken, expiresIn) => {
        const expiresAt = Date.now() + expiresIn * 1000;
        set({ accessToken, refreshToken, expiresAt });
      },

      logout: () => {
        set({
          user: null,
          isAuthenticated: false,
          accessToken: null,
          refreshToken: null,
          expiresAt: null,
        });
      },

      isTokenExpired: () => {
        const { expiresAt } = get();
        if (!expiresAt) return true;
        return Date.now() >= expiresAt;
      },

      loginWithDisplayName: (displayName) => {
        const user: User = {
          id: `local-${Date.now()}`,
          email: '',
          name: displayName,
          provider: 'display_name',
        };
        set({
          user,
          isAuthenticated: true,
          accessToken: null,
          refreshToken: null,
          expiresAt: null,
        });
      },
    }),
    {
      name: 'ffnodes-auth-storage',
    }
  )
);
