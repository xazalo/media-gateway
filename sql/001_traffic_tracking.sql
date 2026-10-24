-- Media Gateway: Traffic tracking tables

-- Daily traffic counters (global)
CREATE TABLE IF NOT EXISTS traffic_daily (
    date DATE NOT NULL,
    metric_type VARCHAR(50) NOT NULL, -- 'bytes', 'downloads', 'requests'
    count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (date, metric_type)
);

-- Monthly traffic counters (global)
CREATE TABLE IF NOT EXISTS traffic_monthly (
    month DATE NOT NULL, -- First day of month
    metric_type VARCHAR(50) NOT NULL,
    count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (month, metric_type)
);

-- Per-user daily traffic counters
CREATE TABLE IF NOT EXISTS traffic_user_daily (
    date DATE NOT NULL,
    user_id UUID NOT NULL,
    metric_type VARCHAR(50) NOT NULL,
    count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (date, user_id, metric_type)
);

-- Traffic limits configuration
CREATE TABLE IF NOT EXISTS traffic_limits (
    id INTEGER PRIMARY KEY DEFAULT 1,
    max_daily_bytes BIGINT NOT NULL DEFAULT 107374182400, -- 100GB
    max_monthly_bytes BIGINT NOT NULL DEFAULT 2199023255552, -- 2TB
    max_daily_downloads BIGINT NOT NULL DEFAULT 50000,
    max_user_daily_bytes BIGINT NOT NULL DEFAULT 10737418240, -- 10GB
    max_user_daily_downloads BIGINT NOT NULL DEFAULT 500,
    max_requests_per_hour INTEGER NOT NULL DEFAULT 5000,
    warning_thresholds JSONB NOT NULL DEFAULT '[75, 90, 95]'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default limits if not exists
INSERT INTO traffic_limits (id) VALUES (1) ON CONFLICT (id) DO NOTHING;

-- Media files metadata (if not exists in main DB)
-- Note: This should reference the existing media table in main DB
-- CREATE TABLE IF NOT EXISTS media (
--     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
--     user_id UUID NOT NULL,
--     key VARCHAR(500) NOT NULL, -- S3 object key
--     filename VARCHAR(255) NOT NULL,
--     size BIGINT NOT NULL,
--     mime_type VARCHAR(100) NOT NULL,
--     is_public BOOLEAN NOT NULL DEFAULT true,
--     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
-- );

-- Index for fast lookups
CREATE INDEX IF NOT EXISTS idx_traffic_daily_date ON traffic_daily(date DESC);
CREATE INDEX IF NOT EXISTS idx_traffic_monthly_month ON traffic_monthly(month DESC);
CREATE INDEX IF NOT EXISTS idx_traffic_user_daily_date ON traffic_user_daily(date DESC);
