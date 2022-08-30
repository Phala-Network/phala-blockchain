# pink-s3

Simple pure Rust AWS S3 Client for Phala network's pink environment.

## Examples

```rust
# fn doctest_ignore() {

use pink_s3 as s3;

let endpoint = "https://s3.ap-southeast-1.amazonaws.com";
let region = "ap-southeast-1";
let access_key = "<Put your S3 access key here>";
let secret_key = "<Put your S3 access secret key here>";

let s3 = s3::S3::new(endpoint, region, access_key, secret_key).unwrap();

let bucket = "my-wallet";
let object_key = "path/to/foo";
let value = b"bar";

s3.put(bucket, object_key, value).unwrap();

let head = s3.head(bucket, object_key).unwrap();
assert_eq!(head.content_length, value.len() as u64);

let v = s3.get(bucket, object_key).unwrap();
assert_eq!(v, value);

s3.delete(bucket, object_key).unwrap();

# }

```

## Supported S3 actions

* HeadObject
* GetObject
* PutObject
* DeleteObject
