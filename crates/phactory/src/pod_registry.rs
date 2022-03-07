use anyhow::Result;
use std::collections::BTreeMap;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct Certificate {
    // The certificate
    pub cert: String,
    // The private key
    pub key: String,
}

pub enum Reply {
    Cert(Certificate),
    Timeout,
}

struct Session {
    mesurement: String,
    reply_channel: Sender<Reply>,
}

pub struct PodRegistry {
    // The Root Certificate Authority used to sign the certificates to be issued to the pods.
    root_ca: String,
    // The waiting pods to be registered in
    sessions: BTreeMap<String, Session>,
    // The URL of the podtracker
    tracker_url: String,
}

impl PodRegistry {
    pub fn register_pod(&mut self, info: &PodInfo) {
        // 1. Do remote attestation
        // 2. Issue certificate
    }

    pub fn start_pod(&mut self, info: &PodInfo) -> Result<()> {
        // 1. Request to the pod tracker to start the pod
        // 2. Wait for register
    }
}


