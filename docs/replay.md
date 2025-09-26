# Tokenomic Replay Utility

## Overview

`replay` is a utility program designed to reproduce the tokenomic computation outside a Gatekeeper. It comes with the following features:

- Fetch chain blocks and Replay the tokenomic computing just in-place without pRuntime needed.
- Provide a HTTP api to query worker's instant computing state.
- Optionally write the workers ming finance events to a PostgreSQL or TimescaleDB database.

## Usage

```
$ ./target/release/replay -h
replay 0.1.0

USAGE:
    replay [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --bind-addr <bind-addr>                    Bind address for local HTTP server. [default: 127.0.0.1:8080]
        --node-uri <node-uri>                      Substrate rpc websocket endpoint. [default: ws://localhost:9944]
        --persist-events-to <persist-events-to>    The PostgresQL database to store the events. [default: ]
        --start-at <start-at>                      The block number to start to replay at. [default: 413895]
```

# Database

To persist the tokenomic event logs to a PostgreSQL compatible database. TimescaleDB is recommended to optimize the performance.

<details>
  <summary>Recommended database DDL</summary>

  ```sql
  DROP TABLE IF EXISTS "worker_finance_events";
  CREATE TABLE "worker_finance_events" (
              "sequence" bigint NOT NULL,
              "pubkey" bytea NOT NULL,
              "block" integer NOT NULL,
              "time" timestamp without time zone NOT NULL,
              "event" text NOT NULL,
              "v" numeric NOT NULL,
              "p" numeric NOT NULL,
              "payout" numeric NOT NULL,
              PRIMARY KEY("time", "sequence")
  ) WITH (oids = false);

  -- If you run it on TimescaleDB, a hypertable can significant optimize the storage and querying
  -- Doc: https://docs.timescale.com/api/latest/hypertable/create_hypertable/#optional-arguments
  SELECT create_hypertable(
              'worker_finance_events',
              'time',
              chunk_time_interval := interval '7 days'
  );
  ```

</details>

To enable the log persistent, set the db url when starting the program:

```
--persist-events-to postgresql://username:password@host:port/database
```

Sample query:

```
select block,time,event,v,payout from worker_finance_events where pubkey = '\xa436ab8f34f73f45b019248bb39b981cd118134afb897fa3c6a4437525ebeb1b' ORDER BY sequence;

 block  |          time           |        event        |        v         |    payout
--------+-------------------------+---------------------+------------------+---------------
 421104 | 2021-09-18 19:01:18.568 | working_started        | 22642.4444767019 |             0
 421159 | 2021-09-18 19:13:48.402 | heartbeat_challenge | 22643.2788294261 |             0
 421163 | 2021-09-18 19:14:54.803 | heartbeat           | 22642.4444767019 |  0.8950341155
 421187 | 2021-09-18 19:20:54.347 | heartbeat_challenge | 22642.7976991938 |             0
 421192 | 2021-09-18 19:22:18.69  | heartbeat           | 22642.4444767019 |  0.4300107155
...
 437708 | 2021-09-21 21:04:18.601 | heartbeat_challenge | 22585.3801870510 |             0
 437718 | 2021-09-21 21:06:42.359 | heartbeat           | 22584.0893745792 |  1.4225321574
 437796 | 2021-09-21 21:24:18.3   | heartbeat_challenge | 22585.2887122098 |             0
 437807 | 2021-09-21 21:26:48.481 | enter_unresponsive  | 22585.4600513300 |             0
 439374 | 2021-09-22 03:31:30.336 | heartbeat           | 22467.7960355975 |             0
 439374 | 2021-09-22 03:31:30.336 | exit_unresponsive   | 22467.7960355975 |             0
 439574 | 2021-09-22 04:19:06.268 | heartbeat_challenge | 22467.9521989589 |             0
```

## State API

The `replay` program exposes a restful api to query the instant state of a worker.

### Individual worker state: `/worker-state/{worker-hex}`

```
curl localhost:8080/worker-state/a436ab8f34f73f45b019248bb39b981cd118134afb897fa3c6a4437525ebeb1b | jq
{
  "current_block": 1923017,
  "total_share": "220881180.46708255354315042496",
  "worker": {
    "bench_state": null,
    "last_gk_responsive_event": 2,
    "last_gk_responsive_event_at_block": 752935,
    "last_heartbeat_at_block": 755816,
    "last_heartbeat_for_block": 755810,
    "working_state": null,
    "registered": true,
    "tokenomic_info": {
      "challenge_time_last": 1637270568284,
      "confidence_level": 4,
      "iteration_last": 746024802,
      "last_payout": "1.0162706988123152853",
      "last_payout_at_block": 755816,
      "last_slash": "0",
      "last_slash_at_block": 752934,
      "p_bench": "2694",
      "p_instant": "2856.98914310795503394445",
      "share": "26976.0464485024567693472",
      "total_payout": "935.2622122582099377903",
      "total_payout_count": 574,
      "total_slash": "364.28754149867309771783",
      "total_slash_count": 21205,
      "v": "26585.9243090089038156205",
      "v_deductible": "56.3328490597386410558",
      "v_init": "22642.44447670198867727826",
      "v_update_at": 1637270640553,
      "v_update_block": 755816
    },
    "unresponsive": false,
    "waiting_heartbeats": []
  }
}
```

### List all the workers states: `/workers`

```
curl localhost:8080/workers | jq
{
  "current_block": 1923021,
  "total_share": "220889860.56347126257605850697",
  "worker": {
    "0x00003800565e748fb0212e44de3732c2176096c4a79cf12b6fea27b580003849": {
      ...(same as "worker-state")
    },
    ...
  }
}
```

### Gatekeeper memory usage estimation: `/meminfo`

```
curl localhost:8080/meminfo | jq
{
  "storage_size": 33051597
}
```
