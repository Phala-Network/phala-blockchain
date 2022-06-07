# Basic Usage
## 1. Grab genesis and headers from the chain
```
headers-cache grab genesis --from-block <number> genesis.bin
headers-cache grab headers --from-block <number> headers.bin
```
## 2. Import the grabbed file into the database.
```
# Kill the ` headers-cache serve` if it is running.
killall headers-cache
headers-cache import genesis genesis.bin
headers-cache import headers headers.bin
```
## 3. Start the cache server
```
ROCKET_PORT=8002 headers-cache serve
```
## 4. Start pherry and tell it to consider the cache server.
```
pherry ... --headers-cache-uri http://localhost:8002
```

# Cache parachain headers and storage changes.
Parachain headers and storage changes can also be cached in `headers-cache` (by #773)
## Grab and import parachain headers
```
headers-cache grab para-headers --from-block <number> para-headers.bin
headers-cache import para-headers para-headers.bin
```
## Grab and import storage changes
```
headers-cache grab storage-changes--from-block <number> storage-changes.bin
headers-cache import storage-changes storage-changes.bin
```

# Trouble shooting
## IO error: While open a file for appending: cache.db/001021.sst: Too many open files
While importing data to the database, the rocksdb would open many files. We can increase the fd limitation by:
```
ulimit -Sn unlimited
```
or
```
sudo ulimit -n unlimited
```
and then try to import again.