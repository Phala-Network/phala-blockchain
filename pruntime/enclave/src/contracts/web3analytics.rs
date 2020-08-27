use crate::std::prelude::v1::*;
use crate::std::vec::Vec;
use crate::std::collections::{HashSet, HashMap};
use serde::{Serialize, Deserialize};
use super::TransactionStatus;

use crate::contracts;
use crate::types::TxRef;

pub type Sid = String;
pub type Timestamp = u32;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PageView {
    id: String,
    sid: Sid,
    cid: String,
    host: String,
    path: String,
    referrer: String,
    ip: String,
    user_agent: String,
    created_at: Timestamp,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OnlineUser {
    sid: Sid,
    cid_count: u32,
    ip_count: u32,
    timestamp: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HourlyStat {
    sid: Sid,
    pv_count: u32,
    cid_count: u32,
    avg_duration: u32,
    timestamp: u32,
}

// contract
#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    SetPageView {
        id: String,
        sid: Sid,
        cid: String,
        host: String,
        path: String,
        referrer: String,
        ip: String,
        user_agent: String,
        created_at: Timestamp,
    },
    GetOnlineUsers {
        start: Timestamp,
        end: Timestamp
    },
    GetHourlyStats {
        start: Timestamp,
        end: Timestamp
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    SetPageView {page_views: Vec<PageView>},
    GetOnlineUsers {online_users: Vec<OnlineUser>},
    GetHourlyStats {hourly_stats: Vec<HourlyStat>}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Web3Analytics {
    page_views: Vec<PageView>,
    online_users: Vec<OnlineUser>,
    hourly_stats: Vec<HourlyStat>
}

impl Web3Analytics {
    pub fn new() -> Self {
        Self {
            page_views: Vec::<PageView>::new(),
            online_users: Vec::<OnlineUser>::new(),
            hourly_stats: Vec::<HourlyStat>::new(),
        }
    }

    pub fn update_online_users(&mut self, start: Timestamp, end: Timestamp) {
        let mut sids = Vec::<Sid>::new();
        let mut cid_map = HashMap::<(Sid, Timestamp), Vec::<String>>::new();
        let mut ip_map = HashMap::<(Sid, Timestamp), Vec::<String>>::new();

        for pv in self.page_views.clone() {
            if pv.created_at <= start {
                continue;
            }

            if pv.created_at > end {
                break;
            }

            if !sids.contains(&pv.sid) {
                sids.push(pv.sid.clone());
            }

            if cid_map.contains_key(&(pv.sid.clone(), pv.created_at)) {
                let mut cids = cid_map.get(&(pv.sid.clone(), pv.created_at)).unwrap().clone();
                if !cids.contains(&pv.cid) {
                    cids.push(pv.cid);
                }
                cid_map.insert((pv.sid.clone(), pv.created_at), cids);
            } else {
                let mut cids = Vec::<String>::new();
                cids.push(pv.cid);
                cid_map.insert((pv.sid.clone(), pv.created_at), cids);
            }

            if ip_map.contains_key(&(pv.sid.clone(), pv.created_at)) {
                let mut ips = ip_map.get(&(pv.sid.clone(), pv.created_at)).unwrap().clone();
                if !ips.contains(&pv.ip) {
                    ips.push(pv.ip);
                }
                ip_map.insert((pv.sid.clone(), pv.created_at), ips);
            } else {
                let mut ips = Vec::<String>::new();
                ips.push(pv.ip);
                ip_map.insert((pv.sid.clone(), pv.created_at), ips);
            }
        }

        self.online_users.clear();

        let mut index = start + 60;
        while index <= end {
            for sid in sids.clone() {
                if !cid_map.contains_key(&(sid.clone(), index)) {
                    continue;
                }

                let cids = cid_map.get(&(sid.clone(), index)).unwrap();
                let ips = ip_map.get(&(sid.clone(), index)).unwrap();
                let ou = OnlineUser {
                    sid,
                    cid_count: cids.len() as u32,
                    ip_count: ips.len() as u32,
                    timestamp: index
                };
                self.online_users.push(ou);
            }
            index += 60;
        }
    }

    pub fn update_hourly_stats(&mut self, start_s: Timestamp, end_s: Timestamp) {
        let mut sids = Vec::<Sid>::new();
        let mut cid_map = HashMap::<(Sid, Timestamp), Vec::<String>>::new();
        let mut pv_count_map = HashMap::<(Sid, Timestamp), u32>::new();

        let start = start_s / 3600 * 3600;
        let end = end_s / 3600 * 3600;

        for pv in self.page_views.clone() {
            if pv.created_at <= start {
                continue;
            }

            if pv.created_at > end {
                break;
            }

            if !sids.contains(&pv.sid) {
                sids.push(pv.sid.clone());
            }

            let ca = pv.created_at / 3600 * 3600;
            if cid_map.contains_key(&(pv.sid.clone(), ca)) {
                let mut cids = cid_map.get(&(pv.sid.clone(), ca)).unwrap().clone();
                if !cids.contains(&pv.cid) {
                    cids.push(pv.cid);
                }
                cid_map.insert((pv.sid.clone(), ca), cids);
            } else {
                let mut cids = Vec::<String>::new();
                cids.push(pv.cid);
                cid_map.insert((pv.sid.clone(), ca), cids);
            }

            if pv_count_map.contains_key(&(pv.sid.clone(), ca)) {
                let pc = pv_count_map.get(&(pv.sid.clone(), ca)).unwrap();
                pv_count_map.insert((pv.sid.clone(), ca), pc + 1);
            } else {
                pv_count_map.insert((pv.sid.clone(), ca), 1);
            }
        }

        self.hourly_stats.clear();

        let mut index = start;
        while index < end {
            for sid in sids.clone() {
                if !cid_map.contains_key(&(sid.clone(), index)) {
                    continue;
                }

                let cids = cid_map.get(&(sid.clone(), index)).unwrap();
                let pc = pv_count_map.get(&(sid.clone(), index)).unwrap();
                let hs = HourlyStat {
                    sid,
                    cid_count: cids.len() as u32,
                    pv_count: *pc,
                    avg_duration: 0,
                    timestamp: index + 3600
                };
                self.hourly_stats.push(hs);
            }
            index += 3600;
        }
    }
}

impl contracts::Contract<Command, Request, Response> for Web3Analytics {
    fn id(&self) -> contracts::ContractId { contracts::W3A }

    fn handle_command(&mut self, _origin: &chain::AccountId, _txref: &TxRef, _cmd: Command) -> TransactionStatus {
        TransactionStatus::Ok
    }

    fn handle_query(&mut self, _origin: Option<&chain::AccountId>, req: Request) -> Response {
        match req {
            Request::SetPageView { id, sid, cid, host, path, referrer, ip, user_agent, created_at } => {
                let pv = PageView {
                    id,
                    sid,
                    cid,
                    host,
                    path,
                    referrer,
                    ip,
                    user_agent,
                    created_at
                };

                let b = self.page_views.clone().into_iter().any(|x| x.id == pv.id);
                if !b {
                    self.page_views.push(pv);
                }

                Response::SetPageView { page_views: self.page_views.clone() }
            }
            Request::GetOnlineUsers { start, end } => {
                self.update_online_users(start, end);
                Response::GetOnlineUsers { online_users: self.online_users.clone() }
            },
            Request::GetHourlyStats { start, end } => {
                self.update_hourly_stats(start, end);
                Response::GetHourlyStats { hourly_stats: self.hourly_stats.clone() }
            },
        }
    }
}
