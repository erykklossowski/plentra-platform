# Plentra Research Intelligence Platform

B2B SaaS energy market analytics tool for Polish wholesale electricity market participants.

## Architecture

```
Browser ──► Vercel (Next.js 15) ──► Railway (Rust/Axum) ──► Stooq CSV APIs
                │                         │
                │ ISR 15min               │ DashMap Cache 15min
                │                         │
           React 19 + Recharts      CORS + JSON responses
```

## Tech Stack

| Layer    | Technology                | Deployment  |
|----------|---------------------------|-------------|
| Backend  | Rust + Axum               | Railway.app |
| Frontend | Next.js 15 + React 19     | Vercel      |
| Styling  | Tailwind CSS v4           | -           |
| Charts   | Recharts                  | -           |
| Data     | Stooq CSV, ENTSO-E (P2)   | -           |

## Local Development

### Prerequisites

- Rust 1.77+ (`rustup update`)
- Node.js 20+ and npm
- (Optional) ENTSO-E API token for Phase 2

### Backend

```bash
cd backend
cp .env.example .env  # if needed
cargo run
# Server starts on http://localhost:8080
# Test: curl http://localhost:8080/health
```

### Frontend

```bash
cd frontend
cp .env.local.example .env.local
npm install
npm run dev
# Opens http://localhost:3000 → redirects to /summary
```

## Environment Variables

### Backend

| Variable          | Default | Description                     |
|-------------------|---------|---------------------------------|
| `PORT`            | `8080`  | Server port                     |
| `RUST_LOG`        | `info`  | Log level                       |
| `ENTSOE_TOKEN`    | -       | ENTSO-E API token (Phase 2)     |
| `CACHE_TTL_FUELS` | `900`   | Fuel cache TTL in seconds       |
| `CACHE_TTL_ENTSOE`| `3600`  | ENTSO-E cache TTL in seconds    |

### Frontend

| Variable               | Default                  | Description         |
|------------------------|--------------------------|---------------------|
| `NEXT_PUBLIC_API_URL`  | `http://localhost:8080`  | Backend API base URL|

## API Endpoints

| Method | Path               | Phase | Description                          |
|--------|--------------------|-------|--------------------------------------|
| GET    | `/health`          | 1     | Health check                         |
| GET    | `/api/fuels`       | 1     | TTF, ARA, EUA prices (live Stooq)    |
| GET    | `/api/spreads`     | 1     | CSS/CDS calculations                 |
| GET    | `/api/summary`     | 1     | Aggregated summary data              |
| GET    | `/api/residual`    | 1*    | Residual demand (mock, live in P2)   |
| GET    | `/api/prices`      | 2     | DA price data                        |
| GET    | `/api/crossborder` | 4     | PL-DE cross-border spreads           |
| GET    | `/api/reserves`    | 2     | Reserve margins                      |
| GET    | `/api/curtailment` | 2     | OZE curtailment data                 |

## Deployment

### Backend (Railway)

1. Connect repo to Railway
2. Set root directory to `backend/`
3. Set env vars: `PORT=8080`, `RUST_LOG=info`
4. Railway auto-detects the Dockerfile

### Frontend (Vercel)

1. Import project in Vercel
2. Set root directory to `frontend/`
3. Set env: `NEXT_PUBLIC_API_URL=https://<railway-url>`
4. Deploy

## Phase Roadmap

| Phase | Screen              | Key Additions                                    |
|-------|---------------------|--------------------------------------------------|
| **1** | Summary             | Scaffold + fuels + CSS/CDS + summary screen      |
| 2     | Stability           | ENTSO-E live data + residual demand + CRI gauge   |
| 3     | Generation Economics| JKZ table + CSS/CDS charts + dispatch signal      |
| 4     | Cross-Border        | PL-DE spread + EU DA prices + EU ranking          |
