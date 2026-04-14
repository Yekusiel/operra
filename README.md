# Operra

Local control-plane desktop app for AWS infrastructure management. Built with Tauri v2, Rust, React, and SQLite.

## Prerequisites

### 1. Install Rust

Download and run the Rust installer from [rustup.rs](https://rustup.rs/).

On Windows, you'll need **Visual Studio Build Tools 2022** with the "Desktop development with C++" workload. The Rust installer will prompt you to install this.

Verify installation:

```bash
rustc --version
cargo --version
```

### 2. Install Node.js

Node.js 18+ is required. Download from [nodejs.org](https://nodejs.org/) or use a version manager.

### 3. Install pnpm

```bash
npm install -g pnpm
```

### 4. System Requirements

- **Windows 11**: WebView2 is included by default
- **Windows 10**: WebView2 may need to be installed from [Microsoft](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

## Setup

```bash
# Clone or navigate to the project
cd operra

# Install frontend dependencies
pnpm install

# Run in development mode (builds Rust backend + starts Vite dev server)
pnpm tauri dev
```

The first build will take 3-5 minutes as it compiles all Rust dependencies (including bundled SQLite). Subsequent builds are incremental and much faster.

## Project Structure

```
operra/
├── src/                    # React frontend (TypeScript)
│   ├── pages/              # Page components (routes)
│   ├── components/         # Reusable UI components
│   ├── hooks/              # React Query hooks
│   └── lib/                # Types and Tauri IPC wrappers
├── src-tauri/              # Rust backend
│   └── src/
│       ├── db/             # SQLite database + migrations
│       ├── models/         # Data models (Project, Scan)
│       ├── commands/       # Tauri IPC command handlers
│       ├── scanner/        # Filesystem scanner + detectors
│       └── adapters/       # AI adapter trait (Claude stub)
```

## Architecture

- **Desktop shell**: Tauri v2
- **Backend**: Rust (commands, database, scanner, adapters)
- **Frontend**: React 18 + TypeScript + Tailwind CSS
- **Data fetching**: TanStack Query v5
- **Database**: SQLite via rusqlite (bundled, WAL mode)
- **IPC**: Tauri v2 commands with serde serialization

All external tool integrations (Claude Code, OpenTofu, AWS CLI) are behind adapter interfaces for clean separation.

## Current Capabilities (Phase 1)

- Create and manage local projects (repo path + AWS config)
- Scan repository to detect languages, frameworks, infrastructure, CI/CD
- Infer application stack type from detected technologies
- View scan results grouped by category with confidence scores
- Persist all data in local SQLite database

## What's Next

- **Phase 2**: Claude Code adapter, architecture questionnaire, infrastructure planning
- **Phase 3**: OpenTofu code generation, deployment review/approval, deployment execution
- **Phase 4**: AWS CloudWatch monitoring, Cost Explorer integration, optimization reports
- **Phase 5**: UX polish, error handling, packaging/distribution

## Development

```bash
# Frontend only (no Rust rebuild)
pnpm dev

# Full app with Rust backend
pnpm tauri dev

# Build production binary
pnpm tauri build

# Check Rust code
cd src-tauri && cargo check
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop | Tauri v2 |
| Backend | Rust |
| Frontend | React 18 + TypeScript |
| Styling | Tailwind CSS 3 |
| Database | SQLite (rusqlite, bundled) |
| Data fetching | TanStack Query v5 |
| Icons | Lucide React |
| Routing | React Router v6 |
