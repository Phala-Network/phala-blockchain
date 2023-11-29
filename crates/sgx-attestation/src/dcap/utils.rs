use alloc::vec;
use alloc::vec::Vec;
use core::time::Duration;
use webpki::types::CertificateDer;
use x509_cert::Certificate;
use asn1_der::{
    typed::{DerDecodable, Sequence},
    DerObject,
};

use crate::dcap::constants::*;
use crate::Error;

pub fn get_intel_extension(der_encoded: &[u8]) -> Result<Vec<u8>, Error> {
    let cert: Certificate = der::Decode::from_der(der_encoded)
        .map_err(|_| Error::IntelExtensionCertificateDecodingError)?;
    let mut extension_iter = cert
        .tbs_certificate
        .extensions
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter(|e| e.extn_id == oids::SGX_EXTENSION)
        .map(|e| e.extn_value.clone());

    let extension = extension_iter.next().ok_or(Error::IntelExtensionAmbiguity)?;
    if extension_iter.next().is_some() {
        //"There should only be one section containing Intel extensions"
        return Err(Error::IntelExtensionAmbiguity);
    }
    Ok(extension.into_bytes())
}

pub fn find_extension(path: &[&[u8]], raw: &[u8]) -> Result<Vec<u8>, Error> {
    let obj = DerObject::decode(raw).map_err(|_| Error::DerDecodingError)?;
    let subobj = get_obj(path, obj)?;
    Ok(subobj.value().to_vec())
}

fn get_obj<'a>(path: &[&[u8]], mut obj: DerObject<'a>) -> Result<DerObject<'a>, Error> {
    for oid in path {
        let seq = Sequence::load(obj).map_err(|_| Error::DerDecodingError )?;
        obj = sub_obj(oid, seq)?;
    }
    Ok(obj)
}

fn sub_obj<'a>(oid: &[u8], seq: Sequence<'a>) -> Result<DerObject<'a>, Error> {
    for i in 0..seq.len() {
        let entry = seq.get(i).map_err(|_| Error::OidIsMissing)?;
        let entry = Sequence::load(entry).map_err(|_| Error::DerDecodingError )?;
        let name = entry.get(0).map_err(|_| Error::OidIsMissing)?;
        let value = entry.get(1).map_err(|_| Error::OidIsMissing)?;
        if name.value() == oid {
            return Ok(value);
        }
    }
    Err(Error::OidIsMissing)
}

pub fn get_fmspc(extension_section: &[u8]) -> Result<Fmspc, Error> {
    let data = find_extension(&[oids::FMSPC.as_bytes()], extension_section)?;
    if data.len() != 6 {
        return Err(Error::FmspcLengthMismatch)
    }

    data.try_into().map_err(|_| Error::FmspcDecodingError)
}

pub fn get_cpu_svn(extension_section: &[u8]) -> Result<CpuSvn, Error> {
    let data = find_extension(&[oids::TCB.as_bytes(), oids::CPUSVN.as_bytes()], extension_section)?;
    if data.len() != 16 {
        return Err(Error::CpuSvnLengthMismatch)
    }

    data.try_into().map_err(|_| Error::CpuSvnDecodingError)
}

pub fn get_pce_svn(extension_section: &[u8]) -> Result<Svn, Error> {
    let data = find_extension(&[oids::TCB.as_bytes(), oids::PCESVN.as_bytes()], extension_section)?;

    match data.len() {
        1 => Ok(u16::from(data[0])),
        2 => Ok(u16::from_be_bytes(data.try_into().map_err(|_| Error::PceSvnDecodingError)?)),
        _ => Err(Error::PceSvnLengthMismatch),
    }
}

pub fn extract_raw_certs(cert_chain: &[u8]) -> Result<Vec<Vec<u8>>, Error> {
    Ok(
        pem::parse_many(cert_chain)
            .map_err(|_| Error::CodecError)?
            .iter()
            .map(|i| i.contents().to_vec() )
            .collect()
    )
}

pub fn extract_certs<'a>(cert_chain: &'a [u8]) -> Result<Vec<CertificateDer<'a>>, Error> {
    let mut certs = Vec::<CertificateDer<'a>>::new();

    let raw_certs = extract_raw_certs(cert_chain)?;
    for raw_cert in raw_certs.iter() {
        let cert = webpki::types::CertificateDer::<'a>::from(raw_cert.to_vec());
        certs.push(cert);
    }

    Ok(certs)
}

/// Encode two 32-byte values in DER format
/// This is meant for 256 bit ECC signatures or public keys
/// TODO: We may could use `asn1_der` crate reimplement this, so we can remove `der` which overlap with `asn1_der`
pub fn encode_as_der(data: &[u8]) -> Result<Vec<u8>, Error> {
    if data.len() != 64 {
        return Err(Error::KeyLengthIsInvalid);
    }
    let mut sequence = der::asn1::SequenceOf::<der::asn1::UintRef, 2>::new();
    sequence
        .add(der::asn1::UintRef::new(&data[0..32]).map_err(|_| Error::PublicKeyIsInvalid)?)
        .map_err(|_| Error::PublicKeyIsInvalid)?;
    sequence
        .add(der::asn1::UintRef::new(&data[32..]).map_err(|_| Error::PublicKeyIsInvalid)?)
        .map_err(|_| Error::PublicKeyIsInvalid)?;
    // 72 should be enough in all cases. 2 + 2 x (32 + 3)
    let mut asn1 = vec![0u8; 72];
    let mut writer = der::SliceWriter::new(&mut asn1);
    writer
        .encode(&sequence)
        .map_err(|_| Error::DerEncodingError)?;
    Ok(writer
        .finish()
        .map_err(|_| Error::DerEncodingError)?
        .to_vec())
}

/// Verifies that the `leaf_cert` in combination with the `intermediate_certs` establishes
/// a valid certificate chain that is rooted in one of the trust anchors that was compiled into to the pallet
pub fn verify_certificate_chain(
    leaf_cert: &webpki::EndEntityCert,
    intermediate_certs: &[CertificateDer],
    verification_time: u64,
) -> Result<(), Error> {
    let time =
        webpki::types::UnixTime::since_unix_epoch(Duration::from_secs(verification_time / 1000));
    let sig_algs = &[webpki::ECDSA_P256_SHA256];
    leaf_cert
        .verify_for_usage(
            sig_algs,
            DCAP_SERVER_ROOTS,
            intermediate_certs,
            time,
            webpki::KeyUsage::server_auth(),
            None,
        )
        .map_err(|_e| Error::CertificateChainIsInvalid)?;

    Ok(())
}
