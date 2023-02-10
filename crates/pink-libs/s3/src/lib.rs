#![doc = include_str!("../README.md")]

use pink::chain_extension::HttpResponse;
use pink_extension as pink;

use scale::{Decode, Encode};
// To encrypt/decrypt HTTP payloads

// To generate AWS4 Signature
use hmac::{Hmac, Mac};
use sha2::Digest;
use sha2::Sha256;

#[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    RequestFailed(u16),
    InvalidEndpoint,
}

/// Infomation of a S3 object
pub struct Head {
    /// The size of the object
    pub content_length: u64,
}

/// The S3 client
pub struct S3<'a> {
    endpoint: &'a str,
    region: &'a str,
    access_key: &'a str,
    secret_key: &'a str,
    virtual_host_mode: bool,
}

impl<'a> S3<'a> {
    /// Create a new S3 client instance
    pub fn new(
        endpoint: &'a str,
        region: &'a str,
        access_key: &'a str,
        secret_key: &'a str,
    ) -> Result<Self, Error> {
        Ok(Self {
            endpoint,
            region,
            access_key,
            secret_key,
            virtual_host_mode: false,
        })
    }

    /// Turn on virtual host mode
    ///
    /// AWS S3 requires virtual host mode for newly created buckets.
    pub fn virtual_host_mode(self) -> Self {
        Self {
            virtual_host_mode: true,
            ..self
        }
    }

    /// Get object metadata from given bucket
    ///
    /// Returns Error::RequestFailed(404) it does not exist.
    pub fn head(&self, bucket_name: &str, object_key: &str) -> Result<Head, Error> {
        let response = self.request("HEAD", bucket_name, object_key, None)?;
        for (k, v) in response.headers {
            if k.to_ascii_lowercase() == "content-length" {
                return Ok(Head {
                    content_length: v.parse().or(Err(Error::RequestFailed(600)))?,
                });
            }
        }
        Err(Error::RequestFailed(response.status_code))
    }

    /// Get object value from bucket `bucket_name` with key `object_key`.
    ///
    /// Returns Error::RequestFailed(404) it does not exist.
    pub fn get(&self, bucket_name: &str, object_key: &str) -> Result<Vec<u8>, Error> {
        Ok(self.request("GET", bucket_name, object_key, None)?.body)
    }

    /// Put an value into bucket `bucket_name` with key `object_key`.
    pub fn put(&self, bucket_name: &str, object_key: &str, value: &[u8]) -> Result<(), Error> {
        self.request("PUT", bucket_name, object_key, Some(value))
            .map(|_| ())
    }

    /// Delete given object from bucket `bucket_name` with key `object_key`.
    ///
    /// Returns Error::RequestFailed(404) it does not exist.
    pub fn delete(&self, bucket_name: &str, object_key: &str) -> Result<(), Error> {
        self.request("DELETE", bucket_name, object_key, None)
            .map(|_| ())
    }

    fn request(
        &self,
        method: &str,
        bucket_name: &str,
        object_key: &str,
        value: Option<&[u8]>,
    ) -> Result<HttpResponse, Error> {
        // Set request values
        let service = "s3";
        let payload_hash = format!("{:x}", Sha256::digest(value.unwrap_or_default()));

        let host = if self.virtual_host_mode {
            format!("{bucket_name}.{}", self.endpoint)
        } else {
            self.endpoint.to_owned()
        };

        // Get current time: datestamp (e.g. 20220727) and amz_date (e.g. 20220727T141618Z)
        let (datestamp, amz_date) = times();

        // 1. Create canonical request
        let canonical_uri = if self.virtual_host_mode {
            format!("/{object_key}")
        } else {
            format!("/{bucket_name}/{object_key}")
        };
        let canonical_querystring = "";
        let canonical_headers = format!(
            "host:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n"
        );
        let signed_headers = "host;x-amz-content-sha256;x-amz-date";
        let canonical_request = format!(
            "{method}\n{canonical_uri}\n{canonical_querystring}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );

        // 2. Create "String to sign"
        let algorithm = "AWS4-HMAC-SHA256";
        let credential_scope = format!("{datestamp}/{}/{service}/aws4_request", self.region);
        let canonical_request_hash = format!("{:x}", Sha256::digest(canonical_request.as_bytes()));
        let string_to_sign = format!(
            "{algorithm}\n{amz_date}\n{credential_scope}\n{canonical_request_hash}"
        );

        // 3. Calculate signature
        let signature_key = get_signature_key(
            self.secret_key.as_bytes(),
            datestamp.as_bytes(),
            self.region.as_bytes(),
            service.as_bytes(),
        );
        let signature_bytes = hmac_sign(&signature_key, string_to_sign.as_bytes());
        let signature = base16::encode_lower(&signature_bytes);

        // 4. Create authorization header
        let authorization_header = format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            algorithm, self.access_key, credential_scope, signed_headers, signature
        );

        let mut headers: Vec<(String, String)> = vec![
            ("Authorization".into(), authorization_header),
            ("x-amz-content-sha256".into(), payload_hash),
            ("x-amz-date".into(), amz_date),
        ];

        let body = if let Some(value) = value {
            headers.push(("Content-Length".into(), format!("{}", &value.len())));
            headers.push(("Content-Type".into(), "binary/octet-stream".into()));
            value
        } else {
            &[]
        };

        // Make HTTP PUT request
        let request_url = format!("https://{host}{canonical_uri}");
        let response = pink::http_req!(method, request_url, body.to_vec(), headers);

        if response.status_code / 100 != 2 {
            return Err(Error::RequestFailed(response.status_code));
        }

        Ok(response)
    }
}

fn times() -> (String, String) {
    // Get block time (UNIX time in nano seconds)and convert to Utc datetime object
    #[cfg(test)]
    let datetime = chrono::Utc::now();
    #[cfg(not(test))]
    let datetime = {
        use chrono::{TimeZone, Utc};
        let time = pink::env().block_timestamp() / 1000;
        Utc.timestamp(time.try_into().unwrap(), 0)
    };

    // Format both date and datetime for AWS4 signature
    let datestamp = datetime.format("%Y%m%d").to_string();
    let datetimestamp = datetime.format("%Y%m%dT%H%M%SZ").to_string();

    (datestamp, datetimestamp)
}

// Create alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

// Returns encrypted hex bytes of key and message using SHA256
fn hmac_sign(key: &[u8], msg: &[u8]) -> Vec<u8> {
    let mut mac =
        <HmacSha256 as Mac>::new_from_slice(key).expect("Could not instantiate HMAC instance");
    mac.update(msg);
    let result = mac.finalize().into_bytes();
    result.to_vec()
}

// Returns the signature key for the complicated version
fn get_signature_key(
    key: &[u8],
    datestamp: &[u8],
    region_name: &[u8],
    service_name: &[u8],
) -> Vec<u8> {
    let k_date = hmac_sign(&[b"AWS4", key].concat(), datestamp);
    let k_region = hmac_sign(&k_date, region_name);
    let k_service = hmac_sign(&k_region, service_name);
    hmac_sign(&k_service, b"aws4_request")
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "can not run concurrently"]
    fn it_works() {
        use crate as s3;

        pink_extension_runtime::mock_ext::mock_all_ext();

        // I don't care to expose them.
        let endpoint = "s3.kvin.wang:8443";
        let region = "garage";
        let access_key = "GKb36294dbfd49a894b19c20cb";
        let secret_key = "c36c43f1ae5bcb27733753a633fb5df82cc57832822275a761d711637bb268d5";

        let s3 = s3::S3::new(endpoint, region, access_key, secret_key)
            .unwrap()
            .virtual_host_mode(); // virtual host mode is required for newly created AWS S3 buckets.

        let bucket = "fat-1";
        let object_key = "path/to/foo";
        let value = b"bar";

        s3.put(bucket, object_key, value).unwrap();

        let head = s3.head(bucket, object_key).unwrap();
        assert_eq!(head.content_length, value.len() as u64);

        let v = s3.get(bucket, object_key).unwrap();
        assert_eq!(v, value);

        s3.delete(bucket, object_key).unwrap();
    }
}
