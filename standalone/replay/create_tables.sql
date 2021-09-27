DROP TABLE IF EXISTS "worker_finance_events";
CREATE TABLE "public"."worker_finance_events" (
    "sequence" bigint NOT NULL,
    "pubkey" bytea NOT NULL,
    "block" integer NOT NULL,
    "time" timestamp without time zone NOT NULL,
    "event" text NOT NULL,
    "v" numeric NOT NULL,
    "p" numeric NOT NULL,
    "payout" numeric NOT NULL
) WITH (oids = false);