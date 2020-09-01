use crate::std::prelude::v1::*;
use crate::std::vec::Vec;
use crate::std::collections::{HashSet, HashMap};
use serde::{Serialize, Deserialize};
use super::TransactionStatus;

use crate::contracts;
use crate::types::TxRef;

pub type Sid = String;
pub type Timestamp = u32;

const MINUTE_IN_SECONDS: u32 = 60;
const HOUR_IN_SECONDS: u32 = 60 * MINUTE_IN_SECONDS;
const DAY_IN_SECONDS: u32 = 24 * HOUR_IN_SECONDS;
const WEEK_IN_SECONDS: u32 = 7 * DAY_IN_SECONDS;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DailyStat {
    stats: Vec<HourlyPageView>,
}

impl DailyStat {
    pub fn new() -> Self {
        Self {
            stats: Vec::<HourlyPageView>::new(),
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
        page_views: Vec<PageView>,
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
    GetDailyStats {
        daily_stat: DailyStat,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    SetPageView {page_views: u32},
    GetOnlineUsers {online_users: Vec<OnlineUser>},
    GetHourlyStats {hourly_stat: HourlyStat},
    GetDailyStats {daily_stat: DailyStat}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Web3Analytics {
    page_views: Vec<PageView>,
    online_users: Vec<OnlineUser>,
    hourly_stat: HourlyStat,
    daily_stat: DailyStat
}

impl Web3Analytics {
    pub fn new() -> Self {
        Self {
            page_views: Vec::<PageView>::new(),
            online_users: Vec::<OnlineUser>::new(),
            hourly_stat: HourlyStat::new(),
            daily_stat: DailyStat::new()
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

            let ca = pv.created_at / MINUTE_IN_SECONDS * MINUTE_IN_SECONDS;
            let mut cids = Vec::<String>::new();
            if cid_map.contains_key(&(pv.sid.clone(), ca)) {
                cids = cid_map.get(&(pv.sid.clone(), ca)).unwrap().clone();
                if !cids.contains(&pv.cid) {
                    cids.push(pv.cid);
                }
            } else {
                cids.push(pv.cid);
            }
            cid_map.insert((pv.sid.clone(), ca), cids);

            let mut ips = Vec::<String>::new();
            if ip_map.contains_key(&(pv.sid.clone(), ca)) {
                ips = ip_map.get(&(pv.sid.clone(), ca)).unwrap().clone();
                if !ips.contains(&pv.ip) {
                    ips.push(pv.ip);
                }
            } else {
                ips.push(pv.ip);
            }
            ip_map.insert((pv.sid.clone(), ca), ips);
        }

        self.online_users.clear();

        let mut index = start + MINUTE_IN_SECONDS;
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
            index += MINUTE_IN_SECONDS;
        }
    }

    pub fn update_hourly_stats(&mut self, start_s: Timestamp, end_s: Timestamp, start_of_week: Timestamp) {
        let mut sids = Vec::<Sid>::new();
        let mut sid_map = HashMap::<Sid, Vec::<String>>::new();
        let mut cid_weekly_map = HashMap::<(Sid, Timestamp), Vec::<String>>::new();
        let mut cid_map = HashMap::<(Sid, Timestamp), Vec::<String>>::new();
        let mut pv_count_map = HashMap::<(Sid, Timestamp), u32>::new();
        let mut path_map = HashMap::<Sid, Vec::<String>>::new();
        let mut path_weekly_map = HashMap::<(Sid, String, Timestamp), u32>::new();
        let mut device_map = HashMap::<Sid, Vec::<String>>::new();
        let mut device_weekly_map = HashMap::<(Sid, String, Timestamp), u32>::new();

        let mut cid_timestamp_map = HashMap::<(String, Timestamp), Vec::<Timestamp>>::new();

        let start = start_s / HOUR_IN_SECONDS * HOUR_IN_SECONDS;
        let end = end_s / HOUR_IN_SECONDS * HOUR_IN_SECONDS;

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

            let ca = pv.created_at / HOUR_IN_SECONDS * HOUR_IN_SECONDS;
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

            let mut tss = Vec::<Timestamp>::new();
            if cid_timestamp_map.contains_key(&(pv.cid.clone(), ca)) {
                tss = cid_timestamp_map.get(&(pv.cid.clone(), ca)).unwrap().clone();
                tss.push(pv.created_at);
            } else {
                tss.push(pv.created_at);
            }
            cid_timestamp_map.insert((pv.cid.clone(), ca), tss);

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

            let diff = (pv.created_at - start_of_week.clone()) / WEEK_IN_SECONDS;
            let date_of_week = start_of_week.clone() + diff * WEEK_IN_SECONDS;
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
                let mut total_duration: u32 = 0;
                for cid in cids.clone() {
                    let tss = cid_timestamp_map.get(&(cid.clone(), index)).unwrap();
                    if tss.len() <= 2 {
                        total_duration += 60;
                    } else {
                        let mut sum:u32 = 0;
                        for i in 1..tss.len() {
                            sum += tss[i] - tss[i-1];
                        }
                        total_duration += sum / (tss.len() as u32 - 1);
                    }
                }

                let avg_duration: u32 = total_duration / (cids.len() as u32);

                let pc = pv_count_map.get(&(sid.clone(), index)).unwrap();
                let hs = HourlyPageView {
                    sid,
                    cid_count: cids.len() as u32,
                    pv_count: *pc,
                    avg_duration,
                    timestamp: index + HOUR_IN_SECONDS
                };
                hpv.push(hs);
            }
            index += HOUR_IN_SECONDS;
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
            index += WEEK_IN_SECONDS;
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
            index += WEEK_IN_SECONDS;
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
            index += WEEK_IN_SECONDS;
        }
        self.hourly_stat.wd = wds;
    }

    fn update_daily_stats(&mut self, daily_stat: DailyStat) {
        let mut sids = Vec::<Sid>::new();
        let mut daily_map = HashMap::<(Sid, Timestamp), (u32, u32, u32)>::new();
        let mut first_date: Timestamp = 0;
        let mut last_date: Timestamp = 0;
        for hourly_stat in daily_stat.stats.clone() {
            let ts = hourly_stat.timestamp;
            let sid = hourly_stat.sid.clone();

            if first_date == 0 {
                first_date = ts;
            }
            last_date = ts;

            if !sids.contains(&sid) {
                sids.push(sid.clone());
            }

            if daily_map.contains_key(&(sid.clone(), ts)) {
                let (pv_count, cid_count, avg_duration) = daily_map.get(&(sid.clone(), ts)).unwrap().clone();
                daily_map.insert((sid.clone(), ts), (hourly_stat.pv_count + pv_count, hourly_stat.cid_count + cid_count, hourly_stat.avg_duration + avg_duration));
            } else {
                daily_map.insert((sid.clone(), ts), (hourly_stat.pv_count, hourly_stat.cid_count, hourly_stat.avg_duration));
            }
        }

        if first_date == 0 {
            return;
        }

        self.daily_stat = DailyStat::new();

        let mut dss = Vec::<HourlyPageView>::new();
        while first_date <= last_date {
            for sid in sids.clone() {
                if daily_map.contains_key(&(sid.clone(), first_date.clone())) {
                    let (pv_count, cid_count, avg_duration) = daily_map.get(&(sid.clone(), first_date.clone())).unwrap().clone();
                    let ds = HourlyPageView {
                        sid,
                        pv_count,
                        cid_count,
                        avg_duration,
                        timestamp: first_date.clone(),
                    };

                    dss.push(ds);
                }
            }
            first_date += DAY_IN_SECONDS;
        }

        self.daily_stat.stats = dss;
    }
}

impl contracts::Contract<Command, Request, Response> for Web3Analytics {
    fn id(&self) -> contracts::ContractId { contracts::WEB3_ANALYTICS }

    fn handle_command(&mut self, _origin: &chain::AccountId, _txref: &TxRef, _cmd: Command) -> TransactionStatus {
        TransactionStatus::Ok
    }

    fn handle_query(&mut self, _origin: Option<&chain::AccountId>, req: Request) -> Response {
        match req {
            Request::SetPageView { page_views } => {
                for page_view in page_views {
                    let b = self.page_views.clone().into_iter().any(|x| x.id == page_view.id);
                    if !b {
                        self.page_views.push(page_view);
                    }
                }

                Response::SetPageView { page_views: self.page_views.len() as u32 }
            }
            Request::GetOnlineUsers { start, end } => {
                self.update_online_users(start, end);
                Response::GetOnlineUsers { online_users: self.online_users.clone() }
            },
            Request::GetHourlyStats { start, end, start_of_week } => {
                self.update_hourly_stats(start, end, start_of_week);
                Response::GetHourlyStats { hourly_stat: self.hourly_stat.clone() }
            },
            Request::GetDailyStats { daily_stat } => {
                self.update_daily_stats(daily_stat);
                Response::GetDailyStats { daily_stat: self.daily_stat.clone() }
            },
        }
    }
}
