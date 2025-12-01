# Phantom Browser - Phase 4

Phantom Browser is a stealth automation browser built with Rust (Tauri) and React.

## New Features (Phase 4)

### Mobile Native Support (Phantom Pocket)
- **Android & iOS**: The core Rhai engine now compiles for mobile targets.
- **UI Adaptation**: Responsive layout with Bottom Tab Bar for mobile.
- **Proxy**: Disabled on mobile (Natural Fingerprinting).

### Remote API (Headless Node)
Control your Headless Desktop instance remotely.

- **Enable**: Run with `--api-port <port>`.
- **Auth**: A Token is printed on startup, or set via `--api-token <token>`.
- **Endpoints**:
    - `GET /health`
    - `GET /logs` (Requires Bearer token)
    - `POST /jobs` (Submit script to run)
    - `POST /stop` (Stop running jobs)

### Embedded Examples ("Starter Pack")
Includes pre-loaded Rhai scripts for common scenarios (Infinite Scroll, Scraper, Login, etc.).

## Development

### Desktop
```bash
pnpm tauri dev
```

### Mobile
(Requires Android Studio / Xcode)
```bash
pnpm tauri android dev
pnpm tauri ios dev
```

### Headless with API
```bash
pnpm tauri dev -- -- --headless --api-port 3000
```
