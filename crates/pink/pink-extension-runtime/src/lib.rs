use std::borrow::Cow;
use std::io::Write;
use std::{
    fmt::Display,
    str::FromStr,
    time::{Duration, SystemTime},
};

use pink_extension::{
    chain_extension::{
        self as ext, HttpRequest, HttpRequestError, HttpResponse, PinkExtBackend, SigType,
        StorageQuotaExceeded,
    },
    Balance, EcdhPublicKey, EcdsaPublicKey, EcdsaSignature, Hash,
};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method,
};
use reqwest_env_proxy::EnvProxyBuilder;
use sp_core::{ByteArray as _, Pair};

pub mod local_cache;
pub mod mock_ext;

pub trait PinkRuntimeEnv {
    type AccountId: AsRef<[u8]> + Display;

    fn address(&self) -> &Self::AccountId;
}

pub struct DefaultPinkExtension<'a, T, Error> {
    pub env: &'a T,
    _e: std::marker::PhantomData<Error>,
}

impl<'a, T, E> DefaultPinkExtension<'a, T, E> {
    pub fn new(env: &'a T) -> Self {
        Self {
            env,
            _e: std::marker::PhantomData,
        }
    }
}

fn block_on<F: core::future::Future>(f: F) -> F::Output {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.block_on(f),
        Err(_) => tokio::runtime::Runtime::new().unwrap().block_on(f),
    }
}

pub fn batch_http_request(
    requests: Vec<HttpRequest>,
    timeout_ms: u64,
) -> Vec<Result<HttpResponse, HttpRequestError>> {
    const MAX_CONCURRENT_REQUESTS: usize = 5;
    if requests.len() > MAX_CONCURRENT_REQUESTS {
        return vec![];
    }
    let futs = requests
        .into_iter()
        .map(|request| async_http_request(request, timeout_ms));
    block_on(futures::future::join_all(futs))
}

pub fn http_request(
    request: HttpRequest,
    timeout_ms: u64,
) -> Result<HttpResponse, HttpRequestError> {
    block_on(async_http_request(request, timeout_ms))
}

async fn async_http_request(
    request: HttpRequest,
    timeout_ms: u64,
) -> Result<HttpResponse, HttpRequestError> {
    if timeout_ms == 0 {
        return Err(HttpRequestError::Timeout);
    }
    let timeout = Duration::from_millis(timeout_ms);
    let url: reqwest::Url = request.url.parse().or(Err(HttpRequestError::InvalidUrl))?;
    let client = reqwest::Client::builder()
        .timeout(timeout)
        .env_proxy(url.host_str().unwrap_or_default())
        .build()
        .or(Err(HttpRequestError::FailedToCreateClient))?;

    let method: Method =
        FromStr::from_str(request.method.as_str()).or(Err(HttpRequestError::InvalidMethod))?;
    let mut headers = HeaderMap::new();
    for (key, value) in &request.headers {
        let key =
            HeaderName::from_str(key.as_str()).or(Err(HttpRequestError::InvalidHeaderName))?;
        let value = HeaderValue::from_str(value).or(Err(HttpRequestError::InvalidHeaderValue))?;
        headers.insert(key, value);
    }

    let result = client
        .request(method, url)
        .headers(headers)
        .body(request.body)
        .send()
        .await;

    let mut response = match result {
        Ok(response) => response,
        Err(err) => {
            // If there is somthing wrong with the network, we can not inspect the reason too
            // much here. Let it return a non-standard 523 here.
            return Ok(HttpResponse {
                status_code: 523,
                reason_phrase: "Unreachable".into(),
                body: format!("{err:?}").into_bytes(),
                headers: vec![],
            });
        }
    };

    let headers: Vec<_> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().into()))
        .collect();

    const MAX_BODY_SIZE: usize = 1024 * 1024 * 2; // 2MB

    let mut body = Vec::new();
    let mut writer = LimitedWriter::new(&mut body, MAX_BODY_SIZE);

    while let Some(chunk) = response
        .chunk()
        .await
        .or(Err(HttpRequestError::NetworkError))?
    {
        writer
            .write_all(&chunk)
            .or(Err(HttpRequestError::ResponseTooLarge))?;
    }

    let response = HttpResponse {
        status_code: response.status().as_u16(),
        reason_phrase: response
            .status()
            .canonical_reason()
            .unwrap_or_default()
            .into(),
        body,
        headers,
    };
    Ok(response)
}

impl<T: PinkRuntimeEnv, E: From<&'static str>> PinkExtBackend for DefaultPinkExtension<'_, T, E> {
    type Error = E;
    fn http_request(&self, request: HttpRequest) -> Result<HttpResponse, Self::Error> {
        http_request(request, 10 * 1000).map_err(|err| err.display().into())
    }

    fn batch_http_request(
        &self,
        requests: Vec<HttpRequest>,
        timeout_ms: u64,
    ) -> Result<Vec<Result<HttpResponse, HttpRequestError>>, Self::Error> {
        Ok(batch_http_request(requests, timeout_ms))
    }

    fn sign(
        &self,
        sigtype: SigType,
        key: Cow<[u8]>,
        message: Cow<[u8]>,
    ) -> Result<Vec<u8>, Self::Error> {
        macro_rules! sign_with {
            ($sigtype:ident) => {{
                let pair = sp_core::$sigtype::Pair::from_seed_slice(&key).or(Err("Invalid key"))?;
                let signature = pair.sign(&message);
                let signature: &[u8] = signature.as_ref();
                signature.to_vec()
            }};
        }

        Ok(match sigtype {
            SigType::Sr25519 => sign_with!(sr25519),
            SigType::Ed25519 => sign_with!(ed25519),
            SigType::Ecdsa => sign_with!(ecdsa),
        })
    }

    fn verify(
        &self,
        sigtype: SigType,
        pubkey: Cow<[u8]>,
        message: Cow<[u8]>,
        signature: Cow<[u8]>,
    ) -> Result<bool, Self::Error> {
        macro_rules! verify_with {
            ($sigtype:ident) => {{
                let pubkey = sp_core::$sigtype::Public::from_slice(&pubkey).map_err(|_| "Invalid public key")?;
                let signature = sp_core::$sigtype::Signature::from_slice(&signature).ok_or("Invalid signature")?;
                Ok(sp_core::$sigtype::Pair::verify(&signature, message, &pubkey))
            }};
        }
        match sigtype {
            SigType::Sr25519 => verify_with!(sr25519),
            SigType::Ed25519 => verify_with!(ed25519),
            SigType::Ecdsa => verify_with!(ecdsa),
        }
    }

    fn derive_sr25519_key(&self, salt: Cow<[u8]>) -> Result<Vec<u8>, Self::Error> {
        // This default implementation is for unit tests. The host should override this.
        let mut seed: <sp_core::sr25519::Pair as Pair>::Seed = Default::default();
        let len = seed.len().min(salt.len());
        seed[..len].copy_from_slice(&salt[..len]);
        let key = sp_core::sr25519::Pair::from_seed(&seed);

        Ok(key.as_ref().secret.to_bytes().to_vec())
    }

    fn get_public_key(&self, sigtype: SigType, key: Cow<[u8]>) -> Result<Vec<u8>, Self::Error> {
        macro_rules! public_key_with {
            ($sigtype:ident) => {{
                sp_core::$sigtype::Pair::from_seed_slice(&key)
                    .or(Err("Invalid key"))?
                    .public()
                    .to_raw_vec()
            }};
        }
        let pubkey = match sigtype {
            SigType::Ed25519 => public_key_with!(ed25519),
            SigType::Sr25519 => public_key_with!(sr25519),
            SigType::Ecdsa => public_key_with!(ecdsa),
        };
        Ok(pubkey)
    }

    fn cache_set(
        &self,
        _key: Cow<[u8]>,
        _value: Cow<[u8]>,
    ) -> Result<Result<(), StorageQuotaExceeded>, Self::Error> {
        Ok(Ok(()))
    }

    fn cache_set_expiration(&self, _key: Cow<[u8]>, _expire: u64) -> Result<(), Self::Error> {
        Ok(())
    }

    fn cache_get(&self, _key: Cow<'_, [u8]>) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(None)
    }

    fn cache_remove(&self, _key: Cow<'_, [u8]>) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(None)
    }

    fn log(&self, level: u8, message: Cow<str>) -> Result<(), Self::Error> {
        let address = self.env.address();
        let level = match level {
            1 => log::Level::Error,
            2 => log::Level::Warn,
            3 => log::Level::Info,
            4 => log::Level::Debug,
            5 => log::Level::Trace,
            _ => log::Level::Error,
        };
        log::log!(target: "pink", level, "[{}] {}", address, message);
        Ok(())
    }

    fn getrandom(&self, length: u8) -> Result<Vec<u8>, Self::Error> {
        let mut buf = vec![0u8; length as _];
        getrandom::getrandom(&mut buf[..]).or(Err("Failed to get random bytes"))?;
        Ok(buf)
    }

    fn is_in_transaction(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn ecdsa_sign_prehashed(
        &self,
        key: Cow<[u8]>,
        message_hash: Hash,
    ) -> Result<EcdsaSignature, Self::Error> {
        let pair = sp_core::ecdsa::Pair::from_seed_slice(&key).or(Err("Invalid key"))?;
        let signature = pair.sign_prehashed(&message_hash);
        Ok(signature.0)
    }

    fn ecdsa_verify_prehashed(
        &self,
        signature: EcdsaSignature,
        message_hash: Hash,
        pubkey: EcdsaPublicKey,
    ) -> Result<bool, Self::Error> {
        let public = sp_core::ecdsa::Public(pubkey);
        let sig = sp_core::ecdsa::Signature(signature);
        Ok(sp_core::ecdsa::Pair::verify_prehashed(
            &sig,
            &message_hash,
            &public,
        ))
    }

    fn system_contract_id(&self) -> Result<ext::AccountId, Self::Error> {
        Err("No default system contract id".into())
    }

    fn balance_of(&self, _account: ext::AccountId) -> Result<(Balance, Balance), Self::Error> {
        Ok((0, 0))
    }

    fn untrusted_millis_since_unix_epoch(&self) -> Result<u64, Self::Error> {
        let duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .or(Err("The system time is earlier than UNIX_EPOCH"))?;
        Ok(duration.as_millis() as u64)
    }

    fn worker_pubkey(&self) -> Result<EcdhPublicKey, Self::Error> {
        Ok(Default::default())
    }

    fn code_exists(&self, _code_hash: Hash, _sidevm: bool) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn import_latest_system_code(
        &self,
        _payer: ext::AccountId,
    ) -> Result<Option<Hash>, Self::Error> {
        Ok(None)
    }

    fn runtime_version(&self) -> Result<(u32, u32), Self::Error> {
        Ok((1, 0))
    }
}

struct LimitedWriter<W> {
    writer: W,
    written: usize,
    limit: usize,
}

impl<W> LimitedWriter<W> {
    fn new(writer: W, limit: usize) -> Self {
        Self {
            writer,
            written: 0,
            limit,
        }
    }
}

impl<W: std::io::Write> std::io::Write for LimitedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.written + buf.len() > self.limit {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Buffer limit exceeded",
            ));
        }
        let wlen = self.writer.write(buf)?;
        self.written += wlen;
        Ok(wlen)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
