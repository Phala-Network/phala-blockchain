# Statistics

## Number of queries and HTTP egress

The message returned by the `/prpc/PhactoryAPI.Statistics` API contains detailed information about the performance and usage of the target worker. The message is organized in a JSON format with several fields providing various metrics.

For example:

```
{
  "uptime": 248,
  "cores": 4,
  "query": {
    "global": {
      "total": 25,
      "dropped": 0,
      "time": 342976
    },
    "by_contract": {
      "0x54a2dc2840e42ecb8891f3246b7aa6964cd78a21954e6c7b20ab58b1b9ba0f04": {
        "total": 19,
        "dropped": 0,
        "time": 2939
      }
    }
  },
  "http_egress": {
    "global": {
      "requests": 26,
      "failures": 13,
      "by_status_code": {
        "404": 12,
        "523": 1,
        "200": 13
      }
    },
    "by_contract": {
      "0x54a2dc2840e42ecb8891f3246b7aa6964cd78a21954e6c7b20ab58b1b9ba0f04": {
        "requests": 26,
        "failures": 13,
        "by_status_code": {
          "523": 1,
          "404": 12,
          "200": 13
        }
      }
    }
  }
}
```

Here's a breakdown of the response fields:

-   uptime: The uptime of the target worker in seconds.
-   cores: The number of cores in the worker.
-   query: Information about the queries made to the worker.
    -   global: This subfield contains global query statistics, including:
        -   total: The total number of queries made to the worker.
        -   dropped: The number of queries that were dropped.
        -   time: The total time spent processing queries, measured in milliseconds.
    -   by_contract: This subfield provides query statistics per contract. Values are the same as the global subfield.
-   http_egress: Information about the HTTP egress from the worker. It is divided into two subfields:
    -   global: This subfield contains global HTTP egress statistics, including:
        -   requests: The total number of HTTP egress requests made by the worker.
        -   failures: The number of failed HTTP egress requests.
        -   by_status_code: A dictionary containing the count of HTTP egress requests by responding status code.
    -   by_contract: This subfield provides HTTP egress statistics per contract. Values are the same as the global subfield.

All of the counters are reset to zero when the worker is restarted. External monitoring systems should take the uptime into account when calculating the rate of queries and HTTP egress.

## Number of users

pRuntime generates an event log for every query made by the user, which includes the user's unique fingerprint. This fingerprint is derived by hashing the user's public key and a salt generated from the cluster's private key. The salt is only accessible within the SGX, ensuring that no one can calculate a user's fingerprint from outside the system. To determine the number of users associated with a particular cluster, external monitoring systems can query the log server periodically to obtain the log. It's important to note that while the fingerprint is likely unique within a cluster. However, a single user would have multiple fingerprints across different clusters.
