# FFNodes Client - Quick Start Guide

## Overview

This is a modernized FFNodes client with Material Design, Bento Grid layout, and OAuth authentication support.

## Features

- ✨ **Material Design 3** - Modern, beautiful UI with Indigo/Pink/Teal color scheme
- 🎯 **Bento Grid Layout** - Dynamic card-based layout throughout the application
- 🔐 **OAuth Authentication** - Login with Google, GitHub, Microsoft, or Facebook
- 🎬 **Movie Posters** - Login page with infinite scrolling TMDB movie posters
- 🚀 **Fast & Responsive** - Built with React, Tauri, and TypeScript
- 🌓 **Light/Dark Mode** - Automatic theme support

## Prerequisites

- **Node.js** 18+ and **pnpm** 8.13.1+
- **Rust** 1.70+ and **Cargo**
- **Tauri CLI** (installed automatically via dev dependencies)

## Installation

### 1. Install Dependencies

```bash
cd ffnodes-client
pnpm install
```

### 2. Configure Environment Variables (Optional for OAuth)

Create a `.env` file in the `ffnodes-client` directory:

```env
# Optional: Add OAuth client IDs (without these, only "Display Name" login works)
VITE_GOOGLE_CLIENT_ID=your_google_client_id
VITE_GITHUB_CLIENT_ID=your_github_client_id
VITE_MICROSOFT_CLIENT_ID=your_microsoft_client_id
VITE_FACEBOOK_CLIENT_ID=your_facebook_app_id
```

**For full OAuth setup, see [OAUTH_SETUP.md](./OAUTH_SETUP.md)**

### 3. Run Development Server

```bash
pnpm tauri-dev
```

This will:
- Start the Vite dev server (frontend)
- Compile the Rust backend
- Launch the Tauri application window

## Usage

### First Run

1. **Login Page** - Choose your authentication method:
   - **OAuth Providers**: Google, GitHub, Microsoft, Facebook
   - **Quick Login**: Enter a display name (no OAuth needed)
   - **QR Code**: Scan to login from mobile

2. **Server Setup** (if using Display Name login):
   - Enter your FFNodes server URL
   - Provide the server GUID (secret code)
   - Test the connection
   - Save configuration

3. **Dashboard** - Main encoding interface with:
   - Current job display (large Bento card)
   - Video queue sidebar
   - Connection status, GPU info, and stats
   - Pause/Resume controls

4. **Settings** - Configure:
   - Server connection details
   - View account information
   - Check GPU details
   - Theme preferences
   - About information

### Authentication Methods

#### Option 1: OAuth Login (Recommended)
- Click any OAuth provider button
- Authorize in your browser
- Automatically redirected back to the app
- Seamless authentication

#### Option 2: Display Name Login (Quick & Simple)
- Click "Skip and use display name"
- Enter any name you want to use
- Proceed to server setup
- No external accounts needed

## Project Structure

```
ffnodes-client/
├── src/
│   ├── components/
│   │   ├── auth/              # Authentication components
│   │   ├── layout/            # Bento Grid components
│   │   ├── ui/                # HeroUI wrapper components
│   │   ├── CurrentJob.tsx     # Encoding job display
│   │   ├── VideoList.tsx      # Job queue
│   │   ├── MoviePosterScroll.tsx  # TMDB poster background
│   │   └── TitleBar.tsx       # Custom window controls
│   ├── pages/
│   │   ├── Login.tsx          # OAuth login page
│   │   ├── Dashboard.tsx      # Main encoding dashboard
│   │   ├── Settings.tsx       # Configuration page
│   │   └── Setup.tsx          # Initial server setup
│   ├── services/
│   │   ├── oauth.ts           # OAuth service
│   │   └── tmdb.ts            # Movie database API
│   ├── stores/
│   │   ├── useAuthStore.ts    # Authentication state
│   │   ├── useConfigStore.ts  # Server configuration
│   │   └── useJobStore.ts     # Encoding jobs state
│   └── types/
│       ├── tmdb.ts            # TMDB type definitions
│       └── iconify.d.ts       # Iconify icon types
├── src-tauri/
│   └── src/
│       ├── oauth.rs           # OAuth backend implementation
│       ├── commands.rs        # Tauri commands
│       └── lib.rs             # Main Rust application
├── OAUTH_SETUP.md             # Detailed OAuth configuration guide
├── QUICKSTART.md              # This file
└── tailwind.config.js         # Material Design theme
```

## Design System

### Colors (Material Design 3)

- **Primary**: Indigo (#3f51b5 light, #7986cb dark)
- **Secondary**: Pink (#e91e63 light, #f48fb1 dark)
- **Tertiary**: Teal (#009688)
- **Background**: #fafafa (light), #121212 (dark)

### Typography

Material Design scale:
- Display (lg/md/sm)
- Headline (lg/md/sm)
- Title (lg/md/sm)
- Body (lg/md/sm)
- Label (lg/md/sm)

### Elevation

Material Design shadows: `shadow-md-1` through `shadow-md-6`

### Components

- **Bento Grid**: Dynamic card layout system
- **Bento Card**: Configurable cards with elevation, backgrounds, and hover effects
- **Card Header/Content/Footer**: Structured card sections

## Building for Production

```bash
pnpm tauri-build
```

This will create installers in `src-tauri/target/release/bundle/`

## Troubleshooting

### Port Already in Use

If you see "Port 1420 is already in use":
```bash
# Find and kill the process using port 1420
# Windows:
netstat -ano | findstr :1420
taskkill /PID <PID> /F

# Linux/Mac:
lsof -i :1420
kill -9 <PID>
```

### OAuth Not Working

1. Check if environment variables are set correctly
2. Ensure redirect URIs are configured in OAuth apps
3. Verify ports 3000-3002 are available
4. See [OAUTH_SETUP.md](./OAUTH_SETUP.md) for detailed troubleshooting

### Build Errors

```bash
# Clear cache and rebuild
pnpm clean
rm -rf node_modules
pnpm install
pnpm tauri-dev
```

### TypeScript Errors

```bash
# Check for TypeScript errors
pnpm exec tsc --noEmit
```

## Development Tips

### Hot Reload

The Vite dev server supports hot module replacement (HMR). Changes to React components will update instantly without restarting.

Rust changes require recompilation - the dev server will automatically rebuild when you save.

### Debugging

- **Frontend**: Open DevTools with `Ctrl+Shift+I` (or `Cmd+Option+I` on Mac)
- **Backend**: Check Rust logs in the terminal running `tauri-dev`
- **Network**: Use browser DevTools Network tab to inspect API calls

### Testing TMDB Integration

The login page uses TMDB API to fetch movie posters. The API key is hardcoded (`378ae44c6e7f5dde094cd8c8456378e0`) as specified.

To test without OAuth:
1. Use "Display Name" login
2. The movie posters will still display
3. You can proceed to server setup

## Scripts

- `pnpm dev` - Start Vite dev server only (frontend)
- `pnpm build` - Build frontend for production
- `pnpm tauri-dev` - Run Tauri app in development mode
- `pnpm tauri-build` - Build Tauri app for production
- `pnpm exec tsc --noEmit` - Check TypeScript errors

## Next Steps

1. ✅ **Basic Setup**: Run the app with display name login
2. 🔐 **Enable OAuth**: Follow [OAUTH_SETUP.md](./OAUTH_SETUP.md) to configure providers
3. 🎨 **Customize Theme**: Modify `tailwind.config.js` for custom colors
4. 🚀 **Deploy**: Build production version with `pnpm tauri-build`

## Resources

- **Tauri Documentation**: https://tauri.app/
- **HeroUI Components**: https://heroui.com/
- **Material Design 3**: https://m3.material.io/
- **TMDB API**: https://www.themoviedb.org/documentation/api
- **OAuth Setup Guide**: [OAUTH_SETUP.md](./OAUTH_SETUP.md)

## Support

For issues or questions:
1. Check the troubleshooting sections in this guide
2. Review [OAUTH_SETUP.md](./OAUTH_SETUP.md) for OAuth-specific issues
3. Check the browser console and terminal logs for errors
4. Verify all dependencies are installed correctly

Enjoy using FFNodes Client! 🎉
