# OAuth Setup Guide for FFNodes Client

This guide will walk you through setting up OAuth authentication for Google, GitHub, Microsoft, and Facebook in your FFNodes client application.

## Table of Contents
- [Overview](#overview)
- [Google OAuth Setup](#google-oauth-setup)
- [GitHub OAuth Setup](#github-oauth-setup)
- [Microsoft OAuth Setup](#microsoft-oauth-setup)
- [Facebook OAuth Setup](#facebook-oauth-setup)
- [Environment Variables](#environment-variables)
- [Testing OAuth Flow](#testing-oauth-flow)
- [Troubleshooting](#troubleshooting)

## Overview

The FFNodes client uses OAuth 2.0 authorization code flow with PKCE for secure authentication. The `tauri-plugin-oauth` handles the OAuth flow by:
1. Opening the provider's authorization URL in the default browser
2. Starting a local HTTP server (on ports 3000, 3001, or 3002) to receive the callback
3. Exchanging the authorization code for access tokens

**Important Security Notes:**
- Client secrets are required for token exchange
- In production, token exchange should happen on your backend server
- Never commit client secrets to version control
- Store secrets in environment variables

---

## Google OAuth Setup

### Step 1: Create a Google Cloud Project

1. Go to the [Google Cloud Console](https://console.cloud.google.com/)
2. Click **"Create Project"** or select an existing project
3. Give your project a name (e.g., "FFNodes Client")
4. Click **"Create"**

### Step 2: Enable Google+ API

1. In the Google Cloud Console, go to **APIs & Services > Library**
2. Search for **"Google+ API"** (or "People API" for newer projects)
3. Click **"Enable"**

### Step 3: Create OAuth 2.0 Credentials

1. Go to **APIs & Services > Credentials**
2. Click **"Create Credentials"** > **"OAuth client ID"**
3. If prompted, configure the OAuth consent screen:
   - Choose **"External"** for user type
   - Fill in app name: "FFNodes Client"
   - Add your email as support email
   - Add authorized domains if needed
   - Add scopes: `email`, `profile`, `openid`
   - Add test users if in testing mode

4. Select **"Desktop app"** as the application type
5. Name it "FFNodes Desktop Client"
6. Click **"Create"**

### Step 4: Configure Redirect URIs

1. Click on your newly created OAuth client
2. Under **"Authorized redirect URIs"**, add:
   ```
   http://localhost:3000
   http://localhost:3001
   http://localhost:3002
   ```
3. Click **"Save"**

### Step 5: Save Credentials

Copy the **Client ID** and **Client Secret** - you'll need these for environment variables.

---

## GitHub OAuth Setup

### Step 1: Create a GitHub OAuth App

1. Go to GitHub Settings: https://github.com/settings/developers
2. Click **"OAuth Apps"** in the left sidebar
3. Click **"New OAuth App"**

### Step 2: Configure the App

Fill in the following:
- **Application name:** `FFNodes Client`
- **Homepage URL:** `https://your-website.com` (or `http://localhost:1420` for development)
- **Application description:** (optional) "Desktop client for FFNodes video encoding"
- **Authorization callback URL:** `http://localhost:3000`

### Step 3: Register the Application

1. Click **"Register application"**
2. On the next page, copy the **Client ID**
3. Click **"Generate a new client secret"**
4. Copy the **Client Secret** immediately (you won't be able to see it again)

### Important Notes:
- GitHub tokens don't expire and cannot be refreshed
- GitHub requires `user:email` scope to access email addresses
- For organizations, you may need additional permissions

---

## Microsoft OAuth Setup

### Step 1: Register an Application in Azure

1. Go to [Azure Portal](https://portal.azure.com/)
2. Navigate to **Azure Active Directory** > **App registrations**
3. Click **"New registration"**

### Step 2: Configure the App

Fill in the following:
- **Name:** `FFNodes Client`
- **Supported account types:** Choose based on your needs:
  - "Accounts in any organizational directory and personal Microsoft accounts" (recommended)
- **Redirect URI:**
  - Platform: **"Public client/native (mobile & desktop)"**
  - URL: `http://localhost:3000`

3. Click **"Register"**

### Step 3: Get Application IDs

1. From the **Overview** page, copy the **Application (client) ID**
2. Copy the **Directory (tenant) ID** (you'll need this for the authorization URL if using a specific tenant)

### Step 4: Create a Client Secret

1. Go to **Certificates & secrets**
2. Click **"New client secret"**
3. Add a description: "FFNodes Desktop Client"
4. Choose an expiration period
5. Click **"Add"**
6. **Immediately copy the secret value** (it won't be shown again)

### Step 5: Configure API Permissions

1. Go to **API permissions**
2. Click **"Add a permission"**
3. Select **"Microsoft Graph"**
4. Select **"Delegated permissions"**
5. Add these permissions:
   - `openid`
   - `profile`
   - `email`
   - `User.Read`
6. Click **"Add permissions"**

### Step 6: Add More Redirect URIs

1. Go to **Authentication**
2. Under **"Platform configurations"**, click on your platform
3. Add additional redirect URIs:
   ```
   http://localhost:3001
   http://localhost:3002
   ```
4. Click **"Save"**

---

## Facebook OAuth Setup

### Step 1: Create a Facebook App

1. Go to [Facebook Developers](https://developers.facebook.com/)
2. Click **"My Apps"** > **"Create App"**
3. Choose **"Consumer"** as the app type
4. Click **"Next"**

### Step 2: Configure Basic Settings

Fill in:
- **App Name:** `FFNodes Client`
- **App Contact Email:** Your email
- Click **"Create App"**

### Step 3: Add Facebook Login Product

1. From the dashboard, find **"Facebook Login"**
2. Click **"Set Up"**
3. Choose **"Other"** (not web, iOS, or Android)

### Step 4: Configure OAuth Settings

1. Go to **Settings** > **Basic**
2. Copy the **App ID** (this is your Client ID)
3. Copy the **App Secret** (this is your Client Secret)
4. Add your app domain if you have one

5. Go to **Facebook Login** > **Settings**
6. Under **"Valid OAuth Redirect URIs"**, add:
   ```
   http://localhost:3000/
   http://localhost:3001/
   http://localhost:3002/
   ```
7. Enable **"Use Strict Mode for Redirect URIs"**
8. Click **"Save Changes"**

### Step 5: Make App Public (Optional)

1. Your app starts in **Development Mode** - only you and added testers can use it
2. To make it public:
   - Complete all required settings
   - Go to **Settings** > **Basic**
   - Switch **"App Mode"** to **"Live"**

### Important Notes:
- Facebook requires HTTPS in production
- You need to add test users in development mode
- Facebook has strict review requirements for accessing email

---

## Environment Variables

Create a `.env` file in the `ffnodes-client` directory (root of the client project):

```env
# Google OAuth
VITE_GOOGLE_CLIENT_ID=your_google_client_id_here
GOOGLE_CLIENT_SECRET=your_google_client_secret_here

# GitHub OAuth
VITE_GITHUB_CLIENT_ID=your_github_client_id_here
GITHUB_CLIENT_SECRET=your_github_client_secret_here

# Microsoft OAuth
VITE_MICROSOFT_CLIENT_ID=your_microsoft_client_id_here
MICROSOFT_CLIENT_SECRET=your_microsoft_client_secret_here

# Facebook OAuth
VITE_FACEBOOK_CLIENT_ID=your_facebook_app_id_here
FACEBOOK_CLIENT_SECRET=your_facebook_app_secret_here
```

**Important:**
- Variables prefixed with `VITE_` are exposed to the frontend (safe for Client IDs)
- Client secrets (without `VITE_` prefix) are only available in the Rust backend
- Add `.env` to your `.gitignore` file to prevent committing secrets
- For production, use secure secret management (Azure Key Vault, AWS Secrets Manager, etc.)

---

## Testing OAuth Flow

### 1. Start the Development Server

```bash
cd ffnodes-client
pnpm tauri-dev
```

### 2. Test Login Flow

1. The app will open and redirect to the login page (if not authenticated)
2. Click on any OAuth provider button (Google, GitHub, Microsoft, or Facebook)
3. Your default browser will open with the provider's authorization page
4. Grant permissions to the app
5. The browser will redirect to `http://localhost:3000` with an authorization code
6. The app will exchange the code for tokens automatically
7. You'll be redirected to the dashboard

### 3. Alternative: Display Name Login

If you don't want to use OAuth, you can:
1. Click **"Skip and use display name"** on the login page
2. Enter a display name
3. Click **"Continue"**
4. Configure your server connection

---

## Troubleshooting

### OAuth Flow Fails to Start

**Error:** "OAuth flow failed: Failed to start HTTP server"

**Solutions:**
- Check if ports 3000, 3001, 3002 are available
- Close any applications using these ports
- Try running with admin privileges

### Redirect URI Mismatch

**Error:** "redirect_uri_mismatch" or "Invalid redirect URI"

**Solutions:**
- Ensure you've added all three redirect URIs to your OAuth app configuration
- Check for typos in the URIs
- Make sure there are no trailing slashes (except for Facebook)
- Wait a few minutes after updating settings (provider caching)

### Token Exchange Fails

**Error:** "Token exchange failed" or "Invalid client secret"

**Solutions:**
- Verify your client secret is correct in the `.env` file
- Check that environment variables are loaded (restart the dev server)
- Ensure the client secret hasn't expired (Microsoft secrets expire)
- Verify the OAuth app is in the correct mode (Facebook: Development vs Live)

### Browser Doesn't Open

**Solutions:**
- Check your default browser settings
- Try manually copying the URL from the console
- Ensure the tauri-plugin-oauth is properly initialized

### "Missing access_token" Error

**Solutions:**
- Check provider-specific requirements (email verification, app review)
- Verify API scopes are correctly configured
- Check if the OAuth app is in development mode and you're not a test user
- Review provider logs/dashboards for specific error messages

### GitHub Email Not Available

**Issue:** GitHub returns `null` for email

**Solutions:**
- Ensure `user:email` scope is requested
- User must have a public email or make their primary email public in GitHub settings
- Use the GitHub API to fetch emails separately if needed

---

## Security Best Practices

1. **Never Commit Secrets**
   - Add `.env` to `.gitignore`
   - Use environment variables for all secrets
   - Rotate secrets regularly

2. **Production Deployment**
   - Move token exchange to a backend server
   - Don't include client secrets in the desktop app
   - Use short-lived access tokens
   - Implement token refresh logic

3. **User Privacy**
   - Only request necessary OAuth scopes
   - Clearly explain what data you're accessing
   - Provide a way to revoke access
   - Store tokens securely (encrypted)

4. **HTTPS in Production**
   - Use HTTPS for redirect URIs in production
   - Facebook and some providers require HTTPS
   - Consider using a custom URL scheme for desktop apps

---

## Additional Resources

- **Google:** [OAuth 2.0 Documentation](https://developers.google.com/identity/protocols/oauth2)
- **GitHub:** [OAuth Apps Documentation](https://docs.github.com/en/developers/apps/building-oauth-apps)
- **Microsoft:** [Microsoft Identity Platform](https://docs.microsoft.com/en-us/azure/active-directory/develop/)
- **Facebook:** [Facebook Login Documentation](https://developers.facebook.com/docs/facebook-login)
- **Tauri Plugin OAuth:** [tauri-plugin-oauth](https://github.com/tauri-apps/tauri-plugin-oauth)

---

## Support

If you encounter issues:
1. Check the provider's developer console for error messages
2. Review the Rust backend logs for detailed error information
3. Verify all redirect URIs are correctly configured
4. Ensure environment variables are properly loaded
5. Test with a simple OAuth flow first (Google is usually easiest)

For more help, refer to the provider-specific documentation linked above.
