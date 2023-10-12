# pink-s3

Simple pure Rust AWS S3 Client for Phala network's pink environment.

## Examples

```rust
# fn doctest_ignore() {

use pink_s3 as s3;

let endpoint = "s3.ap-southeast-1.amazonaws.com";
let region = "ap-southeast-1";
let access_key = "<Put your S3 access key here>";
let secret_key = "<Put your S3 access secret key here>";

let s3 = s3::S3::new(endpoint, region, access_key, secret_key)
    .unwrap()
    .virtual_host_mode(); // virtual host mode is required for newly created AWS S3 buckets.

let bucket = "my-wallet";
let object_key = "path/to/foo";
let value = b"bar";

s3.put(bucket, object_key, value).unwrap();

let head = s3.head(bucket, object_key).unwrap();
let mut head_content_length: u64 = 0;
for (k, v) in head.headers {
    if k.to_ascii_lowercase() == "content-length" {
        head_content_length = v.parse().expect("expect length")
    }
}
assert_eq!(head_content_length, value.len() as u64);

let v = s3.get(bucket, object_key).unwrap().body;
assert_eq!(v, value);

s3.delete(bucket, object_key).unwrap();

# }

```

## Supported S3 actions

* HeadObject
* GetObject
* PutObject
* DeleteObject
