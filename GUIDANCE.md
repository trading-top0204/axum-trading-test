# Stock Trading Backend - Complete Guide

A simple stock trading backend built with **Rust**, **Tokio**, **Axum**, and **PostgreSQL**. Features JWT authentication, portfolio management, and buy/sell order execution.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Project Structure](#project-structure)
4. [Configuration](#configuration)
5. [Database Setup](#database-setup)
6. [API Reference](#api-reference)
7. [Authentication Flow](#authentication-flow)
8. [Architecture Overview](#architecture-overview)
9. [Development & Testing](#development--testing)
10. [Production Deployment](#production-deployment)

---

## Prerequisites

- **Rust** (1.70+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **C compiler/linker** (for compiling Rust dependencies): `sudo apt install build-essential` (Ubuntu/Debian)
- **PostgreSQL** (14+)
- **Docker** (optional, for running PostgreSQL in a container)

---

## Quick Start

### 1. Clone & Enter Project

```bash
cd /home/trading/Workstatioin_trading
```

### 2. Create Environment File

```bash
cp .env.example .env
# Edit .env with your database credentials
```

### 3. Start PostgreSQL (if not running)

**Using Docker:**

```bash
docker run -d \
  --name stock-trading-db \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=stock_trading \
  -p 5432:5432 \
  postgres:16-alpine
```

**Or use existing PostgreSQL:**

```bash
createdb stock_trading
```

### 4. Run Migrations & Start Server

```bash
cargo run
```

Server runs at **http://localhost:3000**

---

## Project Structure

```
.
├── Cargo.toml              # Dependencies
├── build.rs                # Build script for migrations
├── .env.example            # Environment template
├── migrations/
│   └── 001_init.sql        # DB schema & seed data
└── src/
    ├── main.rs             # Entry point, route setup
    ├── config.rs           # Configuration from env
    ├── db.rs               # Database connection pool
    ├── error.rs            # Error types & HTTP responses
    ├── state.rs            # App state (pool, config)
    ├── models/             # Data models
    │   ├── mod.rs
    │   ├── user.rs
    │   ├── order.rs
    │   ├── portfolio.rs
    │   └── stock.rs
    ├── handlers/           # Request handlers
    │   ├── mod.rs
    │   ├── auth.rs         # register, login
    │   └── trading.rs      # portfolio, orders, stocks
    └── middleware/
        └── auth.rs         # JWT extractor
```

---

## Configuration

| Variable          | Description                    | Default                                   |
|-------------------|--------------------------------|-------------------------------------------|
| `DATABASE_URL`    | PostgreSQL connection string   | `postgres://postgres:postgres@localhost:5432/stock_trading` |
| `JWT_SECRET`      | Secret for signing JWT tokens  | `your-secret-key-change-in-production`    |
| `JWT_EXPIRY_HOURS`| Token validity in hours        | `24`                                      |

**⚠️ Important:** Set a strong `JWT_SECRET` in production.

---

## Database Setup

### Schema Summary

- **users** – Email, password hash
- **wallets** – Balance per user (starts at 10,000)
- **stocks** – Symbol, name, current price (seeded: AAPL, GOOGL, MSFT, AMZN, TSLA)
- **portfolio** – User holdings (user_id, symbol, shares)
- **orders** – Buy/sell history

### Run Migrations

Migrations run automatically on startup. To run manually:

```bash
sqlx migrate run
```

---

## API Reference

Base URL: `http://localhost:3000`

### Public Endpoints

#### Health Check

```http
GET /api/health
```

**Response:** `OK`

---

#### Register

```http
POST /api/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "securepassword123"
}
```

**Response:**

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "user_id": "uuid",
  "email": "user@example.com"
}
```

---

#### Login

```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "securepassword123"
}
```

**Response:** Same as Register

---

#### List Stocks

```http
GET /api/stocks
```

**Response:**

```json
[
  {
    "symbol": "AAPL",
    "name": "Apple Inc.",
    "current_price": "175.50"
  }
]
```

---

### Protected Endpoints (require `Authorization: Bearer <token>`)

#### Get Portfolio

```http
GET /api/portfolio
Authorization: Bearer <token>
```

**Response:**

```json
{
  "balance": "10000.00",
  "holdings": [
    {
      "symbol": "AAPL",
      "name": "Apple Inc.",
      "shares": "10",
      "current_price": "175.50",
      "value": "1755.00"
    }
  ]
}
```

---

#### Get Orders

```http
GET /api/orders
Authorization: Bearer <token>
```

**Response:**

```json
[
  {
    "id": "uuid",
    "symbol": "AAPL",
    "order_type": "BUY",
    "shares": "5",
    "price_per_share": "175.50",
    "total_amount": "877.50",
    "created_at": "2025-02-22T10:00:00Z"
  }
]
```

---

#### Place Order

```http
POST /api/orders
Authorization: Bearer <token>
Content-Type: application/json

{
  "symbol": "AAPL",
  "order_type": "BUY",
  "shares": 5
}
```

**Fields:**
- `symbol` – Stock ticker (e.g. AAPL, GOOGL)
- `order_type` – `"BUY"` or `"SELL"`
- `shares` – Number of shares (positive number)

**Response:**

```json
{
  "order_id": "uuid",
  "symbol": "AAPL",
  "order_type": "BUY",
  "shares": "5",
  "price_per_share": "175.50",
  "total_amount": "877.50"
}
```

---

### Error Responses

All errors return JSON:

```json
{
  "error": "Error message"
}
```

| Status | Error             | Description                    |
|--------|-------------------|--------------------------------|
| 401    | Authentication required | Missing or invalid token   |
| 401    | Invalid credentials    | Wrong email/password       |
| 409    | User already exists    | Email taken on register    |
| 400    | Insufficient balance   | Not enough funds for buy   |
| 400    | Insufficient shares    | Not enough shares to sell  |
| 400    | Invalid stock symbol   | Symbol not in catalog      |

---

## Authentication Flow

1. **Register** or **Login** → receive JWT token
2. Send token in all protected requests: `Authorization: Bearer <token>`
3. Token expires after `JWT_EXPIRY_HOURS` (default 24h)
4. On expiry, login again to get a new token

---

## Architecture Overview

- **Tokio** – Async runtime
- **Axum** – HTTP framework, routing, extractors
- **SQLx** – PostgreSQL client, migrations
- **JWT** – Stateless auth via `jsonwebtoken`
- **bcrypt** – Password hashing
- **tower-http** – CORS middleware

Protected routes use the `AuthUser` extractor, which reads the JWT from `Authorization`, validates it, and exposes the user ID to handlers.

---

## Development & Testing

### Run Server

```bash
cargo run
```

### Test with cURL

```bash
# Register
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"pass123"}'

# Use token from response, then:
export TOKEN="<your-token>"

# Get portfolio
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/api/portfolio

# Buy stock
curl -X POST http://localhost:3000/api/orders \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"symbol":"AAPL","order_type":"BUY","shares":5}'
```

---

## Production Deployment

1. **Environment**
   - Set `DATABASE_URL` to your production DB
   - Set a strong `JWT_SECRET` (e.g. 32+ random bytes)
   - Use `JWT_EXPIRY_HOURS` as needed (e.g. 1–24)

2. **Database**
   - Run migrations before deploy
   - Use connection pooling (SQLx pool is already configured)
   - Enable SSL if required by your provider

3. **Build**
   ```bash
   cargo build --release
   ```

4. **Reverse Proxy**
   - Run behind Nginx or Caddy
   - Handle HTTPS and optionally rate limiting

5. **Security**
   - Restrict CORS in production (avoid `Any`)
   - Use HTTPS
   - Keep dependencies updated (`cargo update`)

---

## License

MIT
