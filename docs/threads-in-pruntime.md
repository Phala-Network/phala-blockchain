# Threads in pruntime

| Threads | Number of threads   | Description|
| ------- | --------------- | ------------- |
| main thread | 1  | the entry of the program  |
| benchmark | --cores  | threads to run the benchmark |
| rayon pool | CPU cores  | The rayon is used by wasmer compiler implictly |
| web pool | workers in Rocket.toml  | tokio thread pool serving the web API |
| sidevm pool | --cores + 2  | threads to run sidevm instance |
| tokio blocking pool | max=16 | tokio's helper threads that turn the blocking api into async |

# Additional threads if running in gramine
There are 3 threads runs libOS service
| Thread | Number of threads |
| --------------- | --------------- |
| IPC             | 1               |
| Async events    | 1               |
| TLS-handshaking | 1               |