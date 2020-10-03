Phala Docker Cheatsheet
====

## Commands

### Build image

(Local dev) Software mode

`docker build -f sw.Dockerfile -t phala:dev .`

(Local dev) Hardware mode

`docker build -f hw.Dockerfile --build-arg IAS_SPID='IAS_SPID' --build-arg IAS_API_KEY='IAS_API_KEY' -t phala:dev .`

### Run container

Hardware mode (SGX Driver)

`docker run -ti --device /dev/isgx --name phala -d -p 9944:9944 -p 30333:30333 -p 8000:8000 -v $(pwd)/data:/root/data phala:dev`

Hardware mode (DCAP Driver)

`docker run -ti --device /dev/sgx/enclave --device /dev/sgx/provision --name phala -d -p 9944:9944 -p 30333:30333 -p 8000:8000 -v $(pwd)/data:/root/data phala:dev`

Software mode

`docker run -ti --name phala -d -p 8000:8000 -p 30333:30333 -v $(pwd)/data:/root/data phala:dev`

### Start & stop container

`docker start phala`

`docker stop phala`

### Remove container

`docker kill phala && docker rm phala`

### Show outputs

`docker attach --sig-proxy=false phala`

### Run shell

`docker exec -it phala bash`

### Clean up

`docker image prune`

## Notes

- Modify `dockerfile.d/startup.sh` to suit your needs, you have to rebuild image after change it.
- By default, restart will purge chain, you can disable this behavior in `dockerfile.d/startup.sh`
- Proxy and other Systemd related <https://docs.docker.com/config/daemon/systemd/>
