-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- User wallets (balance for trading)
CREATE TABLE wallets (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    balance DECIMAL(20, 2) NOT NULL DEFAULT 10000.00,  -- Start with 10k demo balance
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Stock symbols (simple catalog)
CREATE TABLE stocks (
    symbol VARCHAR(10) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    current_price DECIMAL(20, 2) NOT NULL
);

-- Seed some sample stocks
INSERT INTO stocks (symbol, name, current_price) VALUES
    ('AAPL', 'Apple Inc.', 175.50),
    ('GOOGL', 'Alphabet Inc.', 140.25),
    ('MSFT', 'Microsoft Corp.', 380.00),
    ('AMZN', 'Amazon.com Inc.', 175.75),
    ('TSLA', 'Tesla Inc.', 245.30);

-- Portfolio (user holdings)
CREATE TABLE portfolio (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    symbol VARCHAR(10) NOT NULL REFERENCES stocks(symbol),
    shares DECIMAL(20, 4) NOT NULL,
    UNIQUE(user_id, symbol)
);

-- Orders (buy/sell history)
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    symbol VARCHAR(10) NOT NULL REFERENCES stocks(symbol),
    order_type VARCHAR(4) NOT NULL CHECK (order_type IN ('BUY', 'SELL')),
    shares DECIMAL(20, 4) NOT NULL,
    price_per_share DECIMAL(20, 2) NOT NULL,
    total_amount DECIMAL(20, 2) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_portfolio_user_id ON portfolio(user_id);
