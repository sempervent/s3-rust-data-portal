# Blacklake UI

A React-based web interface for the Blacklake data artifact management platform.

## Features

- **OIDC Authentication**: Secure login with OpenID Connect
- **Repository Management**: Create, browse, and manage repositories
- **File Operations**: Upload, download, and view files
- **Metadata Browsing**: View Dublin Core metadata and RDF previews
- **Search**: Search across repositories with filters
- **Responsive Design**: Modern UI built with Tailwind CSS and shadcn/ui

## Tech Stack

- **React 18** with TypeScript
- **Vite** for build tooling
- **Tailwind CSS** for styling
- **shadcn/ui** for components
- **React Query** for data fetching
- **Zustand** for state management
- **OIDC Client** for authentication
- **React Router** for navigation

## Getting Started

### Prerequisites

- Node.js 18+ 
- pnpm (recommended) or npm

### Installation

1. Install dependencies:
```bash
pnpm install
```

2. Copy environment configuration:
```bash
cp env.example .env.development
```

3. Update environment variables in `.env.development`:
```env
VITE_API_BASE_URL=http://localhost:8080
VITE_OIDC_ISSUER=http://localhost:8081/realms/master
VITE_OIDC_CLIENT_ID=blacklake
VITE_OIDC_REDIRECT_URI=http://localhost:5173/callback
VITE_OIDC_SCOPE=openid profile email
```

### Development

Start the development server:
```bash
pnpm dev
```

The UI will be available at `http://localhost:5173`.

### Building for Production

```bash
pnpm build
```

The built files will be in the `dist` directory.

## API Integration

The UI communicates with the Blacklake API through the following endpoints:

- `GET /v1/repos` - List repositories
- `POST /v1/repos` - Create repository
- `GET /v1/repos/:repo/tree/:ref` - Get repository tree
- `POST /v1/repos/:repo/upload-init` - Initialize file upload
- `POST /v1/repos/:repo/commit` - Create commit
- `GET /v1/repos/:repo/search` - Search repository
- `GET /v1/repos/:repo/rdf/:ref/:path` - Get RDF metadata

## Authentication

The UI uses OpenID Connect for authentication:

1. User clicks "Sign in with OIDC"
2. Redirected to OIDC provider (Keycloak)
3. After authentication, redirected back to `/callback`
4. Token is stored in memory and used for API requests
5. Automatic token renewal via silent refresh

## Project Structure

```
src/
├── components/          # Reusable UI components
│   └── ui/             # shadcn/ui components
├── hooks/              # Custom React hooks
├── pages/              # Page components
├── stores/             # Zustand stores
├── types/              # TypeScript type definitions
├── utils/              # Utility functions
└── main.tsx           # Application entry point
```

## Development Notes

- The UI expects the Blacklake API to be running on port 8080
- CORS must be enabled in the API for `http://localhost:5173`
- OIDC provider (Keycloak) should be running on port 8081
- All API requests include the Bearer token automatically
- Error handling includes automatic logout on 401 responses
