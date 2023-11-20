use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::borrow::ToOwned;
use core::result::Result;
use core::fmt;
use chrono::{
    DateTime, FixedOffset
};

use crate::Error;

pub(crate) const SGX_ID: &str = "SGX";
pub(crate) const TDX_ID: &str = "TDX";

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TCBVersion {
    V2,
    V3,
    Unsupported { version: u8 }
}

impl fmt::Display for TCBVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TCBVersion::V2 => write!(f, "2"),
            TCBVersion::V3 => write!(f, "3"),
            TCBVersion::Unsupported { version } => write!(f, "{}", version)
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TCBId {
    TDX,
    SGX,
    Unsupported { id: String }
}

impl fmt::Display for TCBId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TCBId::TDX => write!(f, "TDX"),
            TCBId::SGX => write!(f, "SGX"),
            TCBId::Unsupported { id } => write!(f, "{}", id)
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TCBStatus {
    UpToDate,
    OutOfDate,
    ConfigurationNeeded,
    Revoked,
    OutOfDateConfigurationNeeded,
    SWHardeningNeeded,
    ConfigurationAndSWHardeningNeeded,
    Unrecognized { status: Option<String> }
}

impl fmt::Display for TCBStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TCBStatus::UpToDate => write!(f, "UpToDate"),
            TCBStatus::OutOfDate => write!(f, "OutOfDate"),
            TCBStatus::ConfigurationNeeded => write!(f, "ConfigurationNeeded"),
            TCBStatus::Revoked => write!(f, "Revoked"),
            TCBStatus::OutOfDateConfigurationNeeded => write!(f, "OutOfDateConfigurationNeeded"),
            TCBStatus::SWHardeningNeeded => write!(f, "SWHardeningNeeded"),
            TCBStatus::ConfigurationAndSWHardeningNeeded => write!(f, "ConfigurationAndSWHardeningNeeded"),
            TCBStatus::Unrecognized { status } => write!(f, "{:?}", status),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TCBInfo {
    pub version: TCBVersion,
    pub id: TCBId,
    pub issue_date: DateTime<FixedOffset>,
    pub next_update: DateTime<FixedOffset>,
    pub fmspc: Vec<u8>,
    pub pce_id: Vec<u8>,
    pub tcb_type: u32,
    pub tcb_evaluation_data_number: u32,
    pub tcb_levels: Vec<TCBLevel>
    // TODO: tdxModule, we won't support TDX for now
}

impl TCBInfo {
    pub fn from_json_str(json_str: &str) -> Result<Self, Error> {
        let tcb_info: serde_json::Value = serde_json::from_str(json_str).or(Err(Error::CodecError))?;
        let Some(tcb_info) = tcb_info.as_object() else {
            return Err(Error::CodecError)
        };

        let version = {
            let raw_version = tcb_info
                .get("version")
                .ok_or(Error::MissingField { field: "version".to_owned() })?
                .as_u64()
                .ok_or(Error::InvalidFieldValue { field: "version".to_owned() })? as u8;
            match raw_version {
                2 => TCBVersion::V2,
                3 => TCBVersion::V3,
                _ => TCBVersion::Unsupported { version: raw_version }
            }
        };
        if version != TCBVersion::V3 {
            return Err(
                Error::UnsupportedFieldValue { field: "version".to_owned() }
            )
        }
        let id = {
            let raw_id = {
                if version == TCBVersion::V3 {
                    tcb_info
                        .get("id")
                        .ok_or(Error::MissingField { field: "id".to_owned() })?
                        .as_str()
                        .ok_or(Error::InvalidFieldValue { field: "id".to_owned() })?
                } else {
                    SGX_ID
                }
            };

            match raw_id {
                SGX_ID => TCBId::SGX,
                TDX_ID => TCBId::TDX,
                _ => TCBId::Unsupported { id: raw_id.to_owned() }
            }
        };

        if id != TCBId::SGX {
            // TODO: We don't support TDX for now
            return Err(
                Error::UnsupportedFieldValue { field: "id".to_owned() }
            )
        }

        let tcb_type = tcb_info
            .get("tcbType")
            .ok_or(Error::MissingField { field: "tcbType".to_owned() })?
            .as_u64()
            .ok_or(Error::InvalidFieldValue { field: "tcbType".to_owned() })? as u32;
        let tcb_evaluation_data_number = tcb_info
            .get("tcbEvaluationDataNumber")
            .ok_or(Error::MissingField { field: "tcbEvaluationDataNumber".to_owned() })?
            .as_u64()
            .ok_or(Error::InvalidFieldValue { field: "tcbEvaluationDataNumber".to_owned() })? as u32;

        let issue_date = tcb_info
            .get("issueDate")
            .ok_or(Error::MissingField { field: "issueDate".to_owned() })?
            .as_str()
            .ok_or(Error::InvalidFieldValue { field: "issueDate".to_owned() })?;
        let issue_date = chrono::DateTime::parse_from_rfc3339(issue_date)
            .map_err(|_| Error::InvalidFieldValue { field: "issueDate".to_owned() })?;

        let next_update = tcb_info
            .get("nextUpdate")
            .ok_or(Error::MissingField { field: "nextUpdate".to_owned() })?
            .as_str()
            .ok_or(Error::InvalidFieldValue { field: "nextUpdate".to_owned() })?;
        let next_update = chrono::DateTime::parse_from_rfc3339(next_update)
            .map_err(|_| Error::InvalidFieldValue { field: "nextUpdate".to_owned() })?;

        let fmspc = tcb_info
            .get("fmspc")
            .ok_or(Error::MissingField { field: "fmspc".to_owned() })?
            .as_str()
            .ok_or(Error::InvalidFieldValue { field: "fmspc".to_owned() })?;
        let fmspc = hex::decode(fmspc)
            .map_err(|_| Error::InvalidFieldValue { field: "fmspc".to_owned() })?;

        let pce_id = tcb_info
            .get("pceId")
            .ok_or(Error::MissingField { field: "pceId".to_owned() })?
            .as_str()
            .ok_or(Error::InvalidFieldValue { field: "pceId".to_owned() })?;
        let pce_id = hex::decode(pce_id)
            .map_err(|_| Error::InvalidFieldValue { field: "pceId".to_owned() })?;

        // println!("= Parsed TCB info =");
        // println!("Version: {}", version);
        // println!("Id: {}", id);
        // println!("Issue date: {}", issue_date);
        // println!("Next update: {}", next_update);
        // println!("FMSPC: {}", hex::encode(fmspc.clone()));
        // println!("PCE Id: {}", hex::encode(pce_id.clone()));
        // println!("===================");

        let raw_tcb_levels = tcb_info
            .get("tcbLevels")
            .ok_or(Error::MissingField { field: "tcbLevels".to_owned() })?
            .as_array()
            .ok_or(Error::InvalidFieldValue { field: "tcbLevels".to_owned() })?;
        if raw_tcb_levels.is_empty() {
            return Err(
                Error::UnsupportedFieldValue { field: "tcbLevels".to_owned() }
            )
        }

        let mut tcb_levels: Vec<TCBLevel> = Vec::new();
        for raw_tcb_level in raw_tcb_levels {
            let tcb_level = TCBLevel::from_json_value(raw_tcb_level, &version)?;
            tcb_levels.push(tcb_level)
        }

        Ok(
            Self {
                version,
                id,
                issue_date,
                next_update,
                fmspc,
                pce_id,
                tcb_type,
                tcb_evaluation_data_number,
                tcb_levels
            }
        )
    }
}

#[derive(Clone, Debug)]
pub struct TCBLevel {
    pub tcb_status: TCBStatus,
    pub tcb_date: DateTime<FixedOffset>,
    pub pce_svn: u16,
    pub advisory_ids: Vec<String>,
    pub components: [u8; 16],
}

impl TCBLevel {
    pub fn from_json_value(json: &serde_json::Value, _version: &TCBVersion) -> Result<TCBLevel, Error> {
        let tcb_level = json.as_object()
            .ok_or(Error::InvalidFieldValue { field: "tcbLevels".to_owned() })?;

        let tcb_date = tcb_level
            .get("tcbDate")
            .ok_or(Error::MissingField { field: "tcbDate".to_owned() })?
            .as_str()
            .ok_or(Error::InvalidFieldValue { field: "tcbDate".to_owned() })?;
        let tcb_date = chrono::DateTime::parse_from_rfc3339(tcb_date)
            .map_err(|_| Error::InvalidFieldValue { field: "tcbDate".to_owned() })?;

        let advisory_ids = tcb_level
            .get("advisoryIDs")
            .ok_or(Error::MissingField { field: "advisoryIDs".to_owned() })?
            .as_array()
            .ok_or(Error::InvalidFieldValue { field: "advisoryIDs".to_owned() })?;
        let advisory_ids = advisory_ids
            .iter()
            .filter_map(|i| i.as_str().map(|i| i.to_string()))
            .collect();

        let tcb_status = {
            let raw_tcb_status = tcb_level
                .get("tcbStatus")
                .ok_or(Error::MissingField { field: "tcbStatus".to_owned() })?
                .as_str()
                .ok_or(Error::InvalidFieldValue { field: "tcbStatus".to_owned() })?;
            match raw_tcb_status {
                "UpToDate" => TCBStatus::UpToDate,
                "OutOfDate" => TCBStatus::OutOfDate,
                "ConfigurationNeeded" => TCBStatus::ConfigurationNeeded,
                "Revoked" => TCBStatus::Revoked,
                "OutOfDateConfigurationNeeded" => TCBStatus::OutOfDateConfigurationNeeded,
                "SWHardeningNeeded" => TCBStatus::SWHardeningNeeded,
                "ConfigurationAndSWHardeningNeeded" => TCBStatus::ConfigurationAndSWHardeningNeeded,
                _ => TCBStatus::Unrecognized { status: Some(raw_tcb_status.to_owned()) }
            }
        };
        if matches!(tcb_status, TCBStatus::Unrecognized { .. }) {
            return Err(
                Error::UnsupportedFieldValue { field: "tcbStatus".to_owned() }
            )
        }

        let tcb = tcb_level
            .get("tcb")
            .ok_or(Error::MissingField { field: "tcb".to_owned() })?
            .as_object()
            .ok_or(Error::InvalidFieldValue { field: "tcb".to_owned() })?;

        let pce_svn = tcb
            .get("pcesvn")
            .ok_or(Error::MissingField { field: "pcesvn".to_owned() })?
            .as_u64()
            .ok_or(Error::InvalidFieldValue { field: "pcesvn".to_owned() })? as u16;

        let tcb_components = tcb
            .get("sgxtcbcomponents")
            .ok_or(Error::MissingField { field: "sgxtcbcomponents".to_owned() })?
            .as_array()
            .ok_or(Error::InvalidFieldValue { field: "sgxtcbcomponents".to_owned() })?;
        let tcb_components: Vec<_> = tcb_components
            .iter()
            .map(|i| {
                let Some(i) = i.as_object() else {
                    return None
                };
                let Some(i) = i.get("svn") else {
                    return None
                };
                let Some(i) = i.as_u64() else {
                    return None
                };
                Some(i as u8)
            }).collect();
        let mut components = [0u8; 16];
        for i in 0..15 {
            match &tcb_components[i] {
                Some(svn) => {
                    components[i] = *svn;
                },
                None => {
                    return Err(
                        Error::InvalidFieldValue { field: "sgxtcbcomponents".to_owned() }
                    )
                }
            };
        }

        // println!("- Parsed TCB Level -");
        // println!("TCB Status: {}", tcb_status);
        // println!("TCB Date: {}", tcb_date);
        // println!("PCE SVN: {}", pce_svn);
        // println!("Components: {:?}", components);
        // println!("---------------------");

        Ok(
            Self {
                tcb_status,
                tcb_date,
                advisory_ids,
                pce_svn,
                components
            }
        )
    }
}
