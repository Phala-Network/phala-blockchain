# Worker Snapshot Tool

This tool is used to analyze the worker and tokenomic performance. It makes a full dump of the worker status at ever time slice, and then the analyzer is used to produce stats for the workers at different scale.

## Dump the data

Dump the snapshots of all the workers in a time range by `dump-snapshot`. Check the detailed usage:

```
node src/worker-snapshot/dump.js dump-snapshots --help
```

Optionally dump a list of workers in the pools for future analsysis, if you want to break down to the pool level. Check the detailed usage:

```
node src/worker-snapshot/dump.js dump-pool-workers --help
```

## Analyze the data

WIP: documentation

```
node src/worker-snapshot/analyze.js
```
