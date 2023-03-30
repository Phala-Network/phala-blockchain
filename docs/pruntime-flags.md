# Environment variables

| Variable name | Possible values | Allowed in SGX | Description |
| -------- | :------: | :------: | -------: |
| PINK_RUNTIME_PATH  | Path the a FS directory |  No (Always be /pink-runtime) |  The path to search pink runtime library files from |
| RUST_LOG_SANITIZED  | `true`,`false` |  No (Always be true) |  When enable, the pruntime will only logs with targets in the hardcoded whitelist. |
| RUST_LOG  | See doc of env_logger |  Yes |  The log filter expression for the Rust logger |
| all_proxy  | URI prefixed with `socks5://` or `socks5h://` |  Yes |  socks5 proxy for outgoing TCP connections |
| i2p_proxy  | URI prefixed with `socks5://` or `socks5h://` |  Yes |  socks5 proxy for outgoing i2p connections |

# Command line arguments
Run `pruntime -h` for more details
