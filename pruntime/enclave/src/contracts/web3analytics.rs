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
pub struct WeeklySite {
    sid: Sid,
    path: String,
    count: u32,
    timestamp: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeeklyDevice {
    sid: Sid,
    device: String,
    count: u32,
    timestamp: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeeklyClient {
    sid: Sid,
    cids: Vec<String>,
    timestamp: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SiteClient {
    sid: Sid,
    cids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HourlyPageView {
    sid: Sid,
    pv_count: u32,
    cid_count: u32,
    avg_duration: u32,
    timestamp: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HourlyStat {
    hpv: Vec<HourlyPageView>,
    sc: Vec<SiteClient>,
    wc: Vec<WeeklyClient>,
    ws: Vec<WeeklySite>,
    wd: Vec<WeeklyDevice>,
}

impl HourlyStat {
    pub fn new() -> Self {
        Self {
            hpv: Vec::<HourlyPageView>::new(),
            sc: Vec::<SiteClient>::new(),
            wc: Vec::<WeeklyClient>::new(),
            ws: Vec::<WeeklySite>::new(),
            wd: Vec::<WeeklyDevice>::new(),
        }
    }
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
        end: Timestamp,
        start_of_week: Timestamp,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    SetPageView {page_views: Vec<PageView>},
    GetOnlineUsers {online_users: Vec<OnlineUser>},
    GetHourlyStat {hourly_stat: HourlyStat}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Web3Analytics {
    page_views: Vec<PageView>,
    online_users: Vec<OnlineUser>,
    hourly_stat: HourlyStat
}

impl Web3Analytics {
    pub fn new() -> Self {
        Self {
            page_views: Vec::<PageView>::new(),
            online_users: Vec::<OnlineUser>::new(),
            hourly_stat: HourlyStat::new(),
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

    pub fn update_hourly_stats(&mut self, start_s: Timestamp, end_s: Timestamp, start_of_week: Timestamp) {
        const SECONDS_IN_HOUR: u32 = 3600;
        const SECONDS_IN_WEEK: u32 = 7 * 24 * SECONDS_IN_HOUR;

        let mut sids = Vec::<Sid>::new();
        let mut sid_map = HashMap::<Sid, Vec::<String>>::new();
        let mut cid_weekly_map = HashMap::<(Sid, Timestamp), Vec::<String>>::new();
        let mut cid_map = HashMap::<(Sid, Timestamp), Vec::<String>>::new();
        let mut pv_count_map = HashMap::<(Sid, Timestamp), u32>::new();
        let mut path_map = HashMap::<Sid, Vec::<String>>::new();
        let mut path_weekly_map = HashMap::<(Sid, String, Timestamp), u32>::new();
        let mut device_map = HashMap::<Sid, Vec::<String>>::new();
        let mut device_weekly_map = HashMap::<(Sid, String, Timestamp), u32>::new();

        let start = start_s / SECONDS_IN_HOUR * SECONDS_IN_HOUR;
        let end = end_s / SECONDS_IN_HOUR * SECONDS_IN_HOUR;

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

            let ca = pv.created_at / SECONDS_IN_HOUR * SECONDS_IN_HOUR;
            let mut cids = Vec::<String>::new();
            if cid_map.contains_key(&(pv.sid.clone(), ca)) {
                cids = cid_map.get(&(pv.sid.clone(), ca)).unwrap().clone();
                if !cids.contains(&pv.cid) {
                    cids.push(pv.cid.clone());
                }
            } else {
                cids.push(pv.cid.clone());
            }
            cid_map.insert((pv.sid.clone(), ca), cids);

            if pv_count_map.contains_key(&(pv.sid.clone(), ca)) {
                let pc = pv_count_map.get(&(pv.sid.clone(), ca)).unwrap().clone();
                pv_count_map.insert((pv.sid.clone(), ca), pc + 1);
            } else {
                pv_count_map.insert((pv.sid.clone(), ca), 1);
            }

            let mut cids = Vec::<String>::new();
            if sid_map.contains_key(&pv.sid) {
                cids = sid_map.get(&pv.sid).unwrap().clone();
                if !cids.contains(&pv.cid) {
                    cids.push(pv.cid.clone());
                }
            } else {
                cids.push(pv.cid.clone());
            }
            sid_map.insert(pv.sid.clone(), cids);

            let mut paths = Vec::<String>::new();
            if path_map.contains_key(&pv.sid) {
                paths = path_map.get(&pv.sid).unwrap().clone();
                if !paths.contains(&pv.path) {
                    paths.push(pv.path.clone());
                }
            } else {
                paths.push(pv.path.clone());
            }
            path_map.insert(pv.sid.clone(), paths);

            let mut devices = Vec::<String>::new();
            if device_map.contains_key(&pv.sid) {
                devices = device_map.get(&pv.sid).unwrap().clone();
                if !devices.contains(&pv.user_agent) {
                    devices.push(pv.user_agent.clone());
                }
            } else {
                devices.push(pv.user_agent.clone());
            }
            device_map.insert(pv.sid.clone(), devices);

            let diff = (pv.created_at - start_of_week.clone()) / SECONDS_IN_WEEK;
            let date_of_week = start_of_week.clone() + diff * SECONDS_IN_WEEK;
            let mut cids = Vec::<String>::new();
            if cid_weekly_map.contains_key(&(pv.sid.clone(), date_of_week)) {
                cids = cid_weekly_map.get(&(pv.sid.clone(), date_of_week)).unwrap().clone();
                if !cids.contains(&pv.cid) {
                    cids.push(pv.cid.clone());
                }
            } else {
                cids.push(pv.cid.clone());
            }
            cid_weekly_map.insert((pv.sid.clone(), date_of_week), cids);

            if path_weekly_map.contains_key(&(pv.sid.clone(), pv.path.clone(), date_of_week)) {
                let count = path_weekly_map.get(&(pv.sid.clone(), pv.path.clone(), date_of_week)).unwrap().clone();
                path_weekly_map.insert((pv.sid.clone(), pv.path.clone(), date_of_week), count + 1);
            } else {
                path_weekly_map.insert((pv.sid.clone(), pv.path.clone(), date_of_week), 1);
            }

            if device_weekly_map.contains_key(&(pv.sid.clone(), pv.user_agent.clone(), date_of_week)) {
                let count = device_weekly_map.get(&(pv.sid.clone(), pv.user_agent.clone(), date_of_week)).unwrap().clone();
                device_weekly_map.insert((pv.sid.clone(), pv.user_agent.clone(), date_of_week), count + 1);
            } else {
                device_weekly_map.insert((pv.sid.clone(), pv.user_agent.clone(), date_of_week), 1);
            }
        }

        self.hourly_stat = HourlyStat::new();

        let mut hpv = Vec::<HourlyPageView>::new();
        let mut index = start;
        while index < end {
            for sid in sids.clone() {
                if !cid_map.contains_key(&(sid.clone(), index)) {
                    continue;
                }

                let cids = cid_map.get(&(sid.clone(), index)).unwrap().clone();
                let pc = pv_count_map.get(&(sid.clone(), index)).unwrap();
                let hs = HourlyPageView {
                    sid,
                    cid_count: cids.len() as u32,
                    pv_count: *pc,
                    avg_duration: 0,
                    timestamp: index + SECONDS_IN_HOUR
                };
                hpv.push(hs);
            }
            index += SECONDS_IN_HOUR;
        }
        self.hourly_stat.hpv = hpv;

        let mut site_clients = Vec::<SiteClient>::new();
        for sid in sids.clone() {
            let cids = sid_map.get(&sid).unwrap().clone();
            let sc = SiteClient {
                sid,
                cids,
            };

            site_clients.push(sc);
        }
        self.hourly_stat.sc = site_clients;

        let mut wcs = Vec::<WeeklyClient>::new();
        let mut index = start_of_week.clone();
        while index < end {
            for sid in sids.clone() {
                if !cid_weekly_map.contains_key(&(sid.clone(), index)) {
                    continue;
                }

                let cids = cid_weekly_map.get(&(sid.clone(), index)).unwrap().clone();
                let wc = WeeklyClient {
                    sid,
                    cids,
                    timestamp: index
                };
                wcs.push(wc);
            }
            index += SECONDS_IN_WEEK;
        }
        self.hourly_stat.wc = wcs;

        let mut wss = Vec::<WeeklySite>::new();
        index = start_of_week.clone();
        while index < end {
            for sid in sids.clone() {
                let paths = path_map.get(&sid).unwrap().clone();
                for path in paths {
                    if !path_weekly_map.contains_key(&(sid.clone(), path.clone(), index)) {
                        continue;
                    }

                    let count = path_weekly_map.get(&(sid.clone(), path.clone(), index)).unwrap();
                    let ws = WeeklySite {
                        sid: sid.clone(),
                        path,
                        count: *count,
                        timestamp: index
                    };
                    wss.push(ws);
                }
            }
            index += SECONDS_IN_WEEK;
        }
        self.hourly_stat.ws = wss;

        let mut wds = Vec::<WeeklyDevice>::new();
        index = start_of_week.clone();
        while index < end {
            for sid in sids.clone() {
                let devices = device_map.get(&sid).unwrap().clone();
                for device in devices {
                    if !device_weekly_map.contains_key(&(sid.clone(), device.clone(), index)) {
                        continue;
                    }

                    let count = device_weekly_map.get(&(sid.clone(), device.clone(), index)).unwrap();
                    let wd = WeeklyDevice {
                        sid: sid.clone(),
                        device,
                        count: *count,
                        timestamp: index
                    };
                    wds.push(wd);
                }
            }
            index += SECONDS_IN_WEEK;
        }
        self.hourly_stat.wd = wds;
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
            Request::GetHourlyStats { start, end, start_of_week } => {
                self.update_hourly_stats(start, end, start_of_week);
                Response::GetHourlyStat { hourly_stat: self.hourly_stat.clone() }
            },
        }
    }
}
