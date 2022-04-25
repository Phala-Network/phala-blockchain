-- Steps to prepare the database.
-- 1. Install TimescaleDB.
--   docker run -d --name timescaledb -p 5432:5432 -e POSTGRES_PASSWORD=password timescale/timescaledb:latest-pg12
-- 2. Create a database in TimescaleDB.
-- 3. Create table in the database with this SQL scripts.

CREATE EXTENSION IF NOT EXISTS timescaledb;

DROP TABLE IF EXISTS "worker_finance_events";
CREATE TABLE "worker_finance_events" (
    "sequence" bigint NOT NULL UNIQUE,
    "pubkey" bytea NOT NULL,
    "block" integer NOT NULL,
    "time" timestamp without time zone NOT NULL,
    "event" text NOT NULL,
    "v" numeric NOT NULL,
    "p" numeric NOT NULL,
    "payout" numeric NOT NULL
) WITH (oids = false);

-- Doc: https://docs.timescale.com/api/latest/hypertable/create_hypertable/#optional-arguments
SELECT create_hypertable(
    'worker_finance_events',
    'time',
    chunk_time_interval := interval '7 days'
);
