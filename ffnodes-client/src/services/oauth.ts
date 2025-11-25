import { invoke } from '@tauri-apps/api/core';
import { useAuthStore, User } from '../stores/useAuthStore';

export type OAuthProvider = 'google' | 'github' | 'microsoft' | 'facebook';

interface OAuthConfig {
  provider: OAuthProvider;
  clientId: string;
  scope: string[];
  redirectUri: string;
}

// OAuth configuration for each provider
const OAUTH_CONFIGS: Record<OAuthProvider, Omit<OAuthConfig, 'provider'>> = {
  google: {
    clientId: import.meta.env.VITE_GOOGLE_CLIENT_ID || '',
    scope: ['openid', 'email', 'profile'],
    redirectUri: 'http://localhost:3000',
  },
  github: {
    clientId: import.meta.env.VITE_GITHUB_CLIENT_ID || '',
    scope: ['read:user', 'user:email'],
    redirectUri: 'http://localhost:3000',
  },
  microsoft: {
    clientId: import.meta.env.VITE_MICROSOFT_CLIENT_ID || '',
    scope: ['openid', 'email', 'profile'],
    redirectUri: 'http://localhost:3000',
  },
  facebook: {
    clientId: import.meta.env.VITE_FACEBOOK_CLIENT_ID || '',
    scope: ['email', 'public_profile'],
    redirectUri: 'http://localhost:3000',
  },
};

interface OAuthResponse {
  access_token: string;
  refresh_token?: string;
  expires_in: number;
  token_type: string;
  id_token?: string;
}

interface OAuthUserInfo {
  id: string;
  email: string;
  name: string;
  picture?: string;
}

export class OAuthService {
  /**
   * Initiate OAuth flow for a specific provider
   */
  static async login(provider: OAuthProvider): Promise<void> {
    try {
      const config = OAUTH_CONFIGS[provider];

      // Call Tauri backend to start OAuth flow
      const result = await invoke<OAuthResponse>('start_oauth_flow', {
        provider,
        clientId: config.clientId,
        scope: config.scope.join(' '),
        redirectUri: config.redirectUri,
      });

      // Exchange code for tokens with backend API
      const userInfo = await this.exchangeTokenWithBackend(provider, result.access_token);

      // Save tokens and user info
      const authStore = useAuthStore.getState();

      const user: User = {
        id: userInfo.id,
        email: userInfo.email,
        name: userInfo.name,
        avatar: userInfo.picture,
        provider,
      };

      authStore.setUser(user);
      authStore.setTokens(
        result.access_token,
        result.refresh_token || '',
        result.expires_in
      );
    } catch (error) {
      console.error(`OAuth login failed for ${provider}:`, error);
      throw error;
    }
  }

  /**
   * Exchange OAuth token with backend API for session
   */
  private static async exchangeTokenWithBackend(
    provider: OAuthProvider,
    accessToken: string
  ): Promise<OAuthUserInfo> {
    // Get user info from provider
    const userInfo = await this.getUserInfo(provider, accessToken);

    // TODO: Exchange with your backend API
    // const response = await fetch('YOUR_BACKEND_API/auth/oauth', {
    //   method: 'POST',
    //   headers: { 'Content-Type': 'application/json' },
    //   body: JSON.stringify({ provider, accessToken }),
    // });

    return userInfo;
  }

  /**
   * Get user information from OAuth provider
   */
  private static async getUserInfo(
    provider: OAuthProvider,
    accessToken: string
  ): Promise<OAuthUserInfo> {
    const endpoints = {
      google: 'https://www.googleapis.com/oauth2/v2/userinfo',
      github: 'https://api.github.com/user',
      microsoft: 'https://graph.microsoft.com/v1.0/me',
      facebook: 'https://graph.facebook.com/me?fields=id,name,email,picture',
    };

    const response = await fetch(endpoints[provider], {
      headers: {
        Authorization: `Bearer ${accessToken}`,
      },
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch user info from ${provider}`);
    }

    const data = await response.json();

    // Normalize user info across providers
    switch (provider) {
      case 'google':
        return {
          id: data.id,
          email: data.email,
          name: data.name,
          picture: data.picture,
        };
      case 'github':
        return {
          id: data.id.toString(),
          email: data.email,
          name: data.name || data.login,
          picture: data.avatar_url,
        };
      case 'microsoft':
        return {
          id: data.id,
          email: data.mail || data.userPrincipalName,
          name: data.displayName,
          picture: undefined,
        };
      case 'facebook':
        return {
          id: data.id,
          email: data.email,
          name: data.name,
          picture: data.picture?.data?.url,
        };
      default:
        throw new Error(`Unknown provider: ${provider}`);
    }
  }

  /**
   * Refresh access token
   */
  static async refreshToken(): Promise<void> {
    const authStore = useAuthStore.getState();
    const { refreshToken, user } = authStore;

    if (!refreshToken || !user) {
      throw new Error('No refresh token available');
    }

    try {
      const result = await invoke<OAuthResponse>('refresh_oauth_token', {
        provider: user.provider,
        refreshToken,
      });

      authStore.setTokens(
        result.access_token,
        result.refresh_token || refreshToken,
        result.expires_in
      );
    } catch (error) {
      console.error('Token refresh failed:', error);
      authStore.logout();
      throw error;
    }
  }

  /**
   * Logout user
   */
  static logout(): void {
    const authStore = useAuthStore.getState();
    authStore.logout();
  }

  /**
   * Login with display name (simple mode)
   */
  static loginWithDisplayName(displayName: string): void {
    const authStore = useAuthStore.getState();
    authStore.loginWithDisplayName(displayName);
  }
}
