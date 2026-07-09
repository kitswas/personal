-- Personal Finance App — Database Schema
-- Migration 0001: Initial schema
-- All monetary amounts stored as INTEGER (smallest currency unit: paise for INR, cents for USD)
-- Floating-point types are forbidden for financial values (IEEE 754 rounding errors break SUM=0 invariant)

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
    -- Known keys:
    --   'onboarding_complete' = 'true'   (written atomically on wizard finish)
    --   'default_commodity'   = 'INR'    (ISO 4217 code)
);

CREATE TABLE IF NOT EXISTS accounts (
    id        TEXT NOT NULL PRIMARY KEY,   -- UUID v4
    name      TEXT NOT NULL,
    type      TEXT NOT NULL CHECK(type IN ('asset', 'liability', 'equity', 'revenue', 'expense')),
    commodity TEXT NOT NULL                -- ISO 4217 code, e.g. 'INR'
);

CREATE TABLE IF NOT EXISTS transactions (
    id    TEXT NOT NULL PRIMARY KEY,   -- UUID v4
    date  TEXT NOT NULL,               -- ISO 8601 date: YYYY-MM-DD
    payee TEXT NOT NULL,
    notes TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS postings (
    id             TEXT    NOT NULL PRIMARY KEY,                              -- UUID v4
    transaction_id TEXT    NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    account_id     TEXT    NOT NULL REFERENCES accounts(id),
    amount         INTEGER NOT NULL,   -- signed; smallest unit (paise, cents, …)
    commodity      TEXT    NOT NULL    -- ISO 4217
);

-- Invariant enforced in Rust before every INSERT/UPDATE on postings:
--   SELECT SUM(amount) FROM postings WHERE transaction_id = ? MUST equal 0

CREATE INDEX IF NOT EXISTS idx_postings_txn     ON postings(transaction_id);
CREATE INDEX IF NOT EXISTS idx_postings_account ON postings(account_id);
CREATE INDEX IF NOT EXISTS idx_txn_date         ON transactions(date);
