use std::sync::Arc;

use anyhow::Result;
use parity_scale_codec::{Decode, Encode};
use phala_mq::MessageOrigin;

use crate::contracts::{self, NativeContext};
use crate::system::TransactionResult;
use phala_podauth::{QueryError, Request, Response};
use rcgen::{date_time_ymd, BasicConstraints, Certificate, CertificateParams, IsCa};

#[derive(Clone)]
pub struct PodAuth {
    ca: Arc<Certificate>,
}

impl Encode for PodAuth {
    // TODO: implement it
}

impl Decode for PodAuth {
    fn decode<I: parity_scale_codec::Input>(
        _input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        todo!()
    }
}

impl PodAuth {
    pub fn new() -> Result<Self> {
        Ok(Self {
            // TODO: derive the ca from a deterministic source
            ca: Arc::new(new_cert(true)?),
        })
    }
}

impl contracts::NativeContract for PodAuth {
    type Cmd = ();
    type QReq = Request;
    type QResp = Result<Response, QueryError>;

    fn handle_command(
        &mut self,
        _origin: MessageOrigin,
        _cmd: Self::Cmd,
        _context: &mut NativeContext,
    ) -> TransactionResult {
        Ok(Default::default())
    }

    fn handle_query(
        &self,
        origin: Option<&chain::AccountId>,
        req: Request,
        _context: &mut contracts::QueryContext,
    ) -> Result<Response, QueryError> {
        match req {
            Request::Auth { quote } => {
                // TODO: validate quote with DCAP
                let _ = quote;
                let _ = origin;

                let cert = new_cert(false).or(Err(QueryError::MakeCertFailed))?;
                let key = cert.serialize_private_key_pem();
                let cert = cert
                    .serialize_pem_with_signer(&self.ca)
                    .or(Err(QueryError::MakeCertFailed))?;
                Ok(Response::Auth { cert, key })
            }
            Request::GetRootCert => Ok(Response::RootCert {
                cert: self
                    .ca
                    .serialize_pem()
                    .or(Err(QueryError::MakeCertFailed))?,
            }),
        }
    }

    fn snapshot(&self) -> Self {
        self.clone()
    }
}

pub fn new_cert(is_ca: bool) -> Result<Certificate> {
    let mut params: CertificateParams = Default::default();
    params.not_before = date_time_ymd(1975, 01, 01);
    params.not_after = date_time_ymd(4096, 01, 01);
    params.is_ca = if is_ca {
        IsCa::Ca(BasicConstraints::Constrained(1))
    } else {
        IsCa::SelfSignedOnly
    };
    Ok(Certificate::from_params(params)?)
}
