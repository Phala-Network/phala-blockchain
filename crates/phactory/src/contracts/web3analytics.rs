use super::account_id_from_hex;
use super::{NativeContext, TransactionResult};
use crate::contracts::AccountId;
use crate::cryptography::aead;
use std::collections::BTreeMap;
use std::collections::HashMap;
use anyhow::Result;
use core::fmt;
use parity_scale_codec::{Decode, Encode};
use phala_mq::MessageOrigin;

use crate::contracts;
use phala_types::messaging::Web3AnalyticsCommand as Command;

pub type Sid = String;
pub type Timestamp = u32;

const MINUTE_IN_SECONDS: u32 = 60;
const HOUR_IN_SECONDS: u32 = 60 * MINUTE_IN_SECONDS;
const DAY_IN_SECONDS: u32 = 24 * HOUR_IN_SECONDS;
const WEEK_IN_SECONDS: u32 = 7 * DAY_IN_SECONDS;

const KEY: &[u8] =
    &hex_literal::hex!("290c3c5d812a4ba7ce33adf09598a462692a615beb6c80fdafb3f9e3bbef8bc6");

#[derive(Encode, Decode, Debug, Clone)]
pub struct PageView {
    id: String,
    sid: Sid,
    cid: String,
    uid: String,
    host: String,
    path: String,
    referrer: String,
    ip: String,
    user_agent: String,
    created_at: Timestamp,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct OnlineUser {
    sid: Sid,
    cid_count: String,
    ip_count: String,
    timestamp: u32,
}

#[derive(Encode, Decode, Debug, Clone, Default)]
pub struct HourlyPageViewStat {
    sid: Sid,
    pv_count: String,
    cid_count: String,
    avg_duration: String,
    timestamp: u32,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct WeeklySite {
    sid: Sid,
    path: String,
    count: String,
    timestamp: u32,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct WeeklyDevice {
    sid: Sid,
    device: String,
    count: String,
    timestamp: u32,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct WeeklyClient {
    sid: Sid,
    cids: Vec<String>,
    timestamp: u32,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct SiteClient {
    sid: Sid,
    cids: Vec<String>,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct HourlyStat {
    hourly_page_view_stats: Vec<HourlyPageViewStat>,
    site_clients: Vec<SiteClient>,
    weekly_clients: Vec<WeeklyClient>,
    weekly_sites: Vec<WeeklySite>,
    weekly_devices: Vec<WeeklyDevice>,
    total_stats: Vec<HourlyPageViewStat>,
}

impl HourlyStat {
    pub fn new() -> Self {
        Self {
            hourly_page_view_stats: Vec::<HourlyPageViewStat>::new(),
            site_clients: Vec::<SiteClient>::new(),
            weekly_clients: Vec::<WeeklyClient>::new(),
            weekly_sites: Vec::<WeeklySite>::new(),
            weekly_devices: Vec::<WeeklyDevice>::new(),
            total_stats: Vec::<HourlyPageViewStat>::new(),
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct DailyStat {
    stats: Vec<HourlyPageViewStat>,
}

impl DailyStat {
    pub fn new() -> Self {
        Self {
            stats: Vec::<HourlyPageViewStat>::new(),
        }
    }
}

// contract
#[derive(Encode, Decode, Debug)]
pub enum Error {
    NotAuthorized,
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotAuthorized => write!(f, "not authorized"),
            Error::Other(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum Request {
    SetPageView {
        page_views: Vec<PageView>,
        encrypted: bool,
    },
    ClearPageView {
        timestamp: Timestamp,
    },
    GetOnlineUsers {
        start: Timestamp,
        end: Timestamp,
    },
    GetHourlyStats {
        start: Timestamp,
        end: Timestamp,
        start_of_week: Timestamp,
    },
    GetDailyStats {
        daily_stat: DailyStat,
    },
    GetWeeklySites {
        weekly_sites_in_db: Vec<WeeklySite>,
        weekly_sites_new: Vec<WeeklySite>,
    },
    GetWeeklyDevices {
        weekly_devices_in_db: Vec<WeeklyDevice>,
        weekly_devices_new: Vec<WeeklyDevice>,
    },
    GetTotalStat {
        total_stat: HourlyPageViewStat,
        count: String,
    },
    GetConfiguration {
        account: AccountId,
    },
}

#[derive(Encode, Decode, Debug)]
pub enum Response {
    SetPageView {
        page_view_count: u32,
    },
    ClearPageView {
        page_view_count: u32,
    },
    GetOnlineUsers {
        online_users: Vec<OnlineUser>,
        encrypted: bool,
    },
    GetHourlyStats {
        hourly_stat: HourlyStat,
        encrypted: bool,
    },
    GetDailyStats {
        daily_stat: DailyStat,
        encrypted: bool,
    },
    GetWeeklySites {
        weekly_sites: Vec<WeeklySite>,
        encrypted: bool,
    },
    GetWeeklyDevices {
        weekly_devices: Vec<WeeklyDevice>,
        encrypted: bool,
    },
    GetTotalStat {
        total_stat: HourlyPageViewStat,
        encrypted: bool,
    },
    GetConfiguration {
        skip_stat: bool,
    },

    Error(String),
}

pub struct Web3Analytics {
    encrypted: bool,
    page_views: Vec<PageView>,
    online_users: Vec<OnlineUser>,
    hourly_stat: HourlyStat,
    daily_stat: DailyStat,
    weekly_sites: Vec<WeeklySite>,
    weekly_devices: Vec<WeeklyDevice>,
    total_stat: HourlyPageViewStat,

    key: Vec<u8>,
    parser: woothee::parser::Parser,

    no_tracking: BTreeMap<AccountId, bool>,
}

impl Web3Analytics {
    pub fn new() -> Self {
        Self {
            encrypted: false,
            page_views: Vec::<PageView>::new(),
            online_users: Vec::<OnlineUser>::new(),
            hourly_stat: HourlyStat::new(),
            daily_stat: DailyStat::new(),
            weekly_sites: Vec::<WeeklySite>::new(),
            weekly_devices: Vec::<WeeklyDevice>::new(),
            total_stat: HourlyPageViewStat::default(),
            key: KEY.to_owned(),

            parser: woothee::parser::Parser::new(),

            no_tracking: BTreeMap::<AccountId, bool>::new(),
        }
    }

    pub fn update_online_users(&mut self, start: Timestamp, end: Timestamp) {
        let mut sids = Vec::<Sid>::new();
        let mut cid_map = HashMap::<(Sid, Timestamp), Vec<String>>::new();
        let mut ip_map = HashMap::<(Sid, Timestamp), Vec<String>>::new();

        for pv in self.page_views.clone() {
            if pv.created_at < start {
                continue;
            }

            if pv.created_at > end {
                break;
            }

            if !sids.contains(&pv.sid) {
                sids.push(pv.sid.clone());
            }

            let cid = pv.cid;
            let ca = pv.created_at / MINUTE_IN_SECONDS * MINUTE_IN_SECONDS;
            let mut cids = Vec::<String>::new();
            if cid_map.contains_key(&(pv.sid.clone(), ca)) {
                cids = cid_map.get(&(pv.sid.clone(), ca)).unwrap().clone();
                if !cids.contains(&cid) {
                    cids.push(cid);
                }
            } else {
                cids.push(cid);
            }
            cid_map.insert((pv.sid.clone(), ca), cids);

            let ip = if self.encrypted {
                self.decrypt(pv.ip)
            } else {
                pv.ip
            };
            let mut ips = Vec::<String>::new();
            if ip_map.contains_key(&(pv.sid.clone(), ca)) {
                ips = ip_map.get(&(pv.sid.clone(), ca)).unwrap().clone();
                if !ips.contains(&ip) {
                    ips.push(ip);
                }
            } else {
                ips.push(ip);
            }
            ip_map.insert((pv.sid.clone(), ca), ips);
        }

        self.online_users.clear();

        let mut index = start;
        while index < end {
            for sid in sids.clone() {
                if !cid_map.contains_key(&(sid.clone(), index)) {
                    continue;
                }

                let cids = cid_map.get(&(sid.clone(), index)).unwrap();
                let ips = ip_map.get(&(sid.clone(), index)).unwrap();
                let cid_count = if self.encrypted {
                    self.encrypt(cids.len().to_string())
                } else {
                    cids.len().to_string()
                };
                let ip_count = if self.encrypted {
                    self.encrypt(ips.len().to_string())
                } else {
                    ips.len().to_string()
                };
                let ou = OnlineUser {
                    sid,
                    cid_count,
                    ip_count,
                    timestamp: index,
                };
                self.online_users.push(ou);
            }
            index += MINUTE_IN_SECONDS;
        }
    }

    pub fn update_hourly_stats(
        &mut self,
        start_s: Timestamp,
        end_s: Timestamp,
        start_of_week: Timestamp,
    ) {
        let mut sids = Vec::<Sid>::new();
        let mut sid_map = HashMap::<Sid, Vec<String>>::new();
        let mut cid_weekly_map = HashMap::<(Sid, Timestamp), Vec<String>>::new();
        let mut cid_map = HashMap::<(Sid, Timestamp), Vec<String>>::new();
        let mut pv_count_map = HashMap::<(Sid, Timestamp), u32>::new();
        let mut path_map = HashMap::<Sid, Vec<String>>::new();
        let mut path_weekly_map = HashMap::<(Sid, String, Timestamp), u32>::new();
        let mut device_map = HashMap::<Sid, Vec<String>>::new();
        let mut device_weekly_map = HashMap::<(Sid, String, Timestamp), u32>::new();

        let mut cid_timestamp_map = HashMap::<(String, Timestamp), Vec<Timestamp>>::new();

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

            let cid = pv.cid;
            let ca = pv.created_at / HOUR_IN_SECONDS * HOUR_IN_SECONDS;
            let mut cids = Vec::<String>::new();
            if cid_map.contains_key(&(pv.sid.clone(), ca)) {
                cids = cid_map.get(&(pv.sid.clone(), ca)).unwrap().clone();
                if !cids.contains(&cid) {
                    cids.push(cid.clone());
                }
            } else {
                cids.push(cid.clone());
            }
            cid_map.insert((pv.sid.clone(), ca), cids);

            let mut tss = Vec::<Timestamp>::new();
            if cid_timestamp_map.contains_key(&(cid.clone(), ca)) {
                tss = cid_timestamp_map.get(&(cid.clone(), ca)).unwrap().clone();
            }
            tss.push(pv.created_at);
            cid_timestamp_map.insert((cid.clone(), ca), tss);

            if pv_count_map.contains_key(&(pv.sid.clone(), ca)) {
                let pc = *pv_count_map.get(&(pv.sid.clone(), ca)).unwrap();
                pv_count_map.insert((pv.sid.clone(), ca), pc + 1);
            } else {
                pv_count_map.insert((pv.sid.clone(), ca), 1);
            }

            let mut cids = Vec::<String>::new();
            if sid_map.contains_key(&pv.sid) {
                cids = sid_map.get(&pv.sid).unwrap().clone();
                if !cids.contains(&cid) {
                    cids.push(cid.clone());
                }
            } else {
                cids.push(cid.clone());
            }
            sid_map.insert(pv.sid.clone(), cids);

            let path = if self.encrypted {
                self.decrypt(pv.path)
            } else {
                pv.path
            };
            let mut paths = Vec::<String>::new();
            if path_map.contains_key(&pv.sid) {
                paths = path_map.get(&pv.sid).unwrap().clone();
                if !paths.contains(&path) {
                    paths.push(path.clone());
                }
            } else {
                paths.push(path.clone());
            }
            path_map.insert(pv.sid.clone(), paths);

            let user_agent = if self.encrypted {
                self.decrypt(pv.user_agent)
            } else {
                pv.user_agent
            };
            let device = match self.parser.parse(&user_agent) {
                Some(wr) => wr.name.to_string(),
                None => "Unknown".to_string(),
            };
            let mut devices = Vec::<String>::new();
            if device_map.contains_key(&pv.sid) {
                devices = device_map.get(&pv.sid).unwrap().clone();
                if !devices.contains(&device) {
                    devices.push(device.clone());
                }
            } else {
                devices.push(device.clone());
            }
            device_map.insert(pv.sid.clone(), devices);

            let diff = (pv.created_at - start_of_week) / WEEK_IN_SECONDS;
            let date_of_week = start_of_week + diff * WEEK_IN_SECONDS;
            let mut cids = Vec::<String>::new();
            if cid_weekly_map.contains_key(&(pv.sid.clone(), date_of_week)) {
                cids = cid_weekly_map
                    .get(&(pv.sid.clone(), date_of_week))
                    .unwrap()
                    .clone();
                if !cids.contains(&cid) {
                    cids.push(cid.clone());
                }
            } else {
                cids.push(cid.clone());
            }
            cid_weekly_map.insert((pv.sid.clone(), date_of_week), cids);

            if path_weekly_map.contains_key(&(pv.sid.clone(), path.clone(), date_of_week)) {
                let count = *path_weekly_map
                    .get(&(pv.sid.clone(), path.clone(), date_of_week))
                    .unwrap();
                path_weekly_map.insert((pv.sid.clone(), path.clone(), date_of_week), count + 1);
            } else {
                path_weekly_map.insert((pv.sid.clone(), path.clone(), date_of_week), 1);
            }

            if device_weekly_map.contains_key(&(pv.sid.clone(), device.clone(), date_of_week)) {
                let count = *device_weekly_map
                    .get(&(pv.sid.clone(), device.clone(), date_of_week))
                    .unwrap();
                device_weekly_map.insert((pv.sid.clone(), device.clone(), date_of_week), count + 1);
            } else {
                device_weekly_map.insert((pv.sid.clone(), device.clone(), date_of_week), 1);
            }
        }

        self.hourly_stat = HourlyStat::new();

        let mut hpv = Vec::<HourlyPageViewStat>::new();
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
                        let mut sum: u32 = 0;
                        for i in 1..tss.len() {
                            sum += tss[i] - tss[i - 1];
                        }
                        total_duration += sum / (tss.len() as u32 - 1);
                    }
                }

                let avg_duration_str = (total_duration / (cids.len() as u32)).to_string();

                let pc = pv_count_map.get(&(sid.clone(), index)).unwrap();
                let cid_count = if self.encrypted {
                    self.encrypt(cids.len().to_string())
                } else {
                    cids.len().to_string()
                };
                let pv_count = if self.encrypted {
                    self.encrypt((*pc).to_string())
                } else {
                    (*pc).to_string()
                };
                let avg_duration = if self.encrypted {
                    self.encrypt(avg_duration_str)
                } else {
                    avg_duration_str
                };
                let hs = HourlyPageViewStat {
                    sid,
                    cid_count,
                    pv_count,
                    avg_duration,
                    timestamp: index,
                };
                hpv.push(hs);
            }
            index += HOUR_IN_SECONDS;
        }
        self.hourly_stat.hourly_page_view_stats = hpv;

        let mut site_clients = Vec::<SiteClient>::new();
        for sid in sids.clone() {
            let cids = sid_map.get(&sid).unwrap().clone();
            let sc = SiteClient { sid, cids };

            site_clients.push(sc);
        }
        self.hourly_stat.site_clients = site_clients;

        let mut wcs = Vec::<WeeklyClient>::new();
        let mut index = start_of_week;
        while index < end {
            for sid in sids.clone() {
                if !cid_weekly_map.contains_key(&(sid.clone(), index)) {
                    continue;
                }

                let cids = cid_weekly_map.get(&(sid.clone(), index)).unwrap().clone();
                let wc = WeeklyClient {
                    sid,
                    cids,
                    timestamp: index,
                };
                wcs.push(wc);
            }
            index += WEEK_IN_SECONDS;
        }
        self.hourly_stat.weekly_clients = wcs;

        let mut wss = Vec::<WeeklySite>::new();
        index = start_of_week;
        while index < end {
            for sid in sids.clone() {
                let paths = path_map.get(&sid).unwrap().clone();
                for p in paths {
                    if !path_weekly_map.contains_key(&(sid.clone(), p.clone(), index)) {
                        continue;
                    }

                    let count = path_weekly_map
                        .get(&(sid.clone(), p.clone(), index))
                        .unwrap();
                    let path = if self.encrypted {
                        self.encrypt(p.clone())
                    } else {
                        p.clone()
                    };
                    let count = if self.encrypted {
                        self.encrypt((*count).to_string())
                    } else {
                        (*count).to_string()
                    };
                    let ws = WeeklySite {
                        sid: sid.clone(),
                        path,
                        count,
                        timestamp: index,
                    };
                    wss.push(ws);
                }
            }
            index += WEEK_IN_SECONDS;
        }
        self.hourly_stat.weekly_sites = wss;

        let mut wds = Vec::<WeeklyDevice>::new();
        index = start_of_week;
        while index < end {
            for sid in sids.clone() {
                let devices = device_map.get(&sid).unwrap().clone();
                for dev in devices {
                    if !device_weekly_map.contains_key(&(sid.clone(), dev.clone(), index)) {
                        continue;
                    }

                    let count = device_weekly_map
                        .get(&(sid.clone(), dev.clone(), index))
                        .unwrap();
                    let device = if self.encrypted {
                        self.encrypt(dev.clone())
                    } else {
                        dev.clone()
                    };
                    let count = if self.encrypted {
                        self.encrypt((*count).to_string())
                    } else {
                        (*count).to_string()
                    };
                    let wd = WeeklyDevice {
                        sid: sid.clone(),
                        device,
                        count,
                        timestamp: index,
                    };
                    wds.push(wd);
                }
            }
            index += WEEK_IN_SECONDS;
        }
        self.hourly_stat.weekly_devices = wds;
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

            let pv_count = if self.encrypted {
                self.decrypt(hourly_stat.pv_count)
            } else {
                hourly_stat.pv_count
            };
            let cid_count = if self.encrypted {
                self.decrypt(hourly_stat.cid_count)
            } else {
                hourly_stat.cid_count
            };
            let avg_duration = if self.encrypted {
                self.decrypt(hourly_stat.avg_duration)
            } else {
                hourly_stat.avg_duration
            };
            let mut hourly_stat_pv_count = pv_count.parse::<u32>().unwrap();
            let mut hourly_stat_cid_count = cid_count.parse::<u32>().unwrap();
            let mut hourly_stat_avg_duration = avg_duration.parse::<u32>().unwrap();
            if daily_map.contains_key(&(sid.clone(), ts)) {
                let (pv_count0, cid_count0, avg_duration0) =
                    *daily_map.get(&(sid.clone(), ts)).unwrap();
                hourly_stat_pv_count += pv_count0;
                hourly_stat_cid_count += cid_count0;
                hourly_stat_avg_duration += avg_duration0;
            }
            daily_map.insert(
                (sid.clone(), ts),
                (
                    hourly_stat_pv_count,
                    hourly_stat_cid_count,
                    hourly_stat_avg_duration,
                ),
            );
        }

        if first_date == 0 {
            return;
        }

        self.daily_stat = DailyStat::new();

        let mut dss = Vec::<HourlyPageViewStat>::new();
        while first_date <= last_date {
            for sid in sids.clone() {
                if daily_map.contains_key(&(sid.clone(), first_date)) {
                    let (pv_count, cid_count, avg_duration) =
                        daily_map.get(&(sid.clone(), first_date)).unwrap();
                    let ds = HourlyPageViewStat {
                        sid,
                        pv_count: if self.encrypted {
                            self.encrypt(pv_count.to_string())
                        } else {
                            pv_count.to_string()
                        },
                        cid_count: if self.encrypted {
                            self.encrypt(cid_count.to_string())
                        } else {
                            cid_count.to_string()
                        },
                        avg_duration: if self.encrypted {
                            self.encrypt(avg_duration.to_string())
                        } else {
                            avg_duration.to_string()
                        },
                        timestamp: first_date,
                    };

                    dss.push(ds);
                }
            }
            first_date += DAY_IN_SECONDS;
        }

        self.daily_stat.stats = dss;
    }

    fn update_weekly_sites(
        &mut self,
        weekly_sites_in_db: Vec<WeeklySite>,
        weekly_sites_new: Vec<WeeklySite>,
    ) {
        self.weekly_sites.clear();

        for ws in weekly_sites_new {
            let path = if self.encrypted {
                self.decrypt(ws.path.clone())
            } else {
                ws.path.clone()
            };
            let mut matched = false;
            for ws_db in weekly_sites_in_db.clone() {
                let path_db = if self.encrypted {
                    self.decrypt(ws_db.path.clone())
                } else {
                    ws_db.path.clone()
                };
                if ws.sid == ws_db.sid && ws.timestamp == ws_db.timestamp && path == path_db {
                    matched = true;
                    let count = if self.encrypted {
                        self.decrypt(ws.count.clone()).parse::<u32>().unwrap()
                    } else {
                        ws.count.clone().parse::<u32>().unwrap()
                    };
                    let count_db = if self.encrypted {
                        self.decrypt(ws_db.count).parse::<u32>().unwrap()
                    } else {
                        ws_db.count.parse::<u32>().unwrap()
                    };
                    let total = if self.encrypted {
                        self.encrypt((count + count_db).to_string())
                    } else {
                        (count + count_db).to_string()
                    };
                    let w = WeeklySite {
                        sid: ws.sid.clone(),
                        count: total,
                        path: ws.path.clone(),
                        timestamp: ws.timestamp,
                    };
                    self.weekly_sites.push(w);

                    break;
                }
            }

            if !matched {
                self.weekly_sites.push(ws);
            }
        }
    }

    fn update_weekly_devices(
        &mut self,
        weekly_devices_in_db: Vec<WeeklyDevice>,
        weekly_devices_new: Vec<WeeklyDevice>,
    ) {
        self.weekly_devices.clear();

        for wd in weekly_devices_new {
            let device = if self.encrypted {
                self.decrypt(wd.device.clone())
            } else {
                wd.device.clone()
            };
            let mut matched = false;
            for wd_db in weekly_devices_in_db.clone() {
                let device_db = if self.encrypted {
                    self.decrypt(wd_db.device.clone())
                } else {
                    wd_db.device.clone()
                };
                if wd.sid == wd_db.sid && wd.timestamp == wd_db.timestamp && device == device_db {
                    matched = true;
                    let count = if self.encrypted {
                        self.decrypt(wd.count.clone()).parse::<u32>().unwrap()
                    } else {
                        wd.count.clone().parse::<u32>().unwrap()
                    };
                    let count_db = if self.encrypted {
                        self.decrypt(wd_db.count).parse::<u32>().unwrap()
                    } else {
                        wd_db.count.parse::<u32>().unwrap()
                    };
                    let total = if self.encrypted {
                        self.encrypt((count + count_db).to_string())
                    } else {
                        (count + count_db).to_string()
                    };
                    let w = WeeklyDevice {
                        sid: wd.sid.clone(),
                        count: total,
                        device: wd.device.clone(),
                        timestamp: wd.timestamp,
                    };
                    self.weekly_devices.push(w);

                    break;
                }
            }

            if !matched {
                self.weekly_devices.push(wd);
            }
        }
    }

    fn update_total_stat(&mut self, total_stat: HourlyPageViewStat, count_str: String) {
        self.total_stat = HourlyPageViewStat::default();

        if !self.encrypted {
            return;
        }

        let mut count = self.decrypt(total_stat.pv_count).parse::<u32>().unwrap();
        if !count_str.is_empty() {
            count += self.decrypt(count_str).parse::<u32>().unwrap();
        }

        let avg_duration = self
            .decrypt(total_stat.avg_duration)
            .parse::<u32>()
            .unwrap()
            / 2;

        self.total_stat = HourlyPageViewStat {
            sid: total_stat.sid,
            cid_count: total_stat.cid_count,
            pv_count: self.encrypt(count.to_string()),
            avg_duration: self.encrypt(avg_duration.to_string()),
            timestamp: total_stat.timestamp,
        }
    }

    fn encrypt(&mut self, data: String) -> String {
        let mut msg = data.as_bytes().to_vec();
        let iv = crate::generate_random_iv();
        aead::encrypt(&iv, &self.key, &mut msg);

        format!("{:}|{:}", base64::encode(&iv), base64::encode(&msg))
    }

    fn decrypt(&mut self, data: String) -> String {
        let v: Vec<&str> = data.split('|').collect();
        let iv_b64 = v[0];
        let cipher_b64 = v[1];

        let iv = base64::decode(&iv_b64).unwrap();
        let mut cipher_data = base64::decode(&cipher_b64).unwrap();

        let cid = aead::decrypt(&iv, &*self.key, &mut cipher_data);
        String::from_utf8(cid.to_vec()).unwrap()
    }
}

impl contracts::NativeContract for Web3Analytics {
    type Cmd = Command;
    type QReq = Request;
    type QResp = Response;

    fn id(&self) -> contracts::ContractId {
        contracts::id256(contracts::WEB3_ANALYTICS)
    }

    fn handle_command(
        &mut self,
        _context: &NativeContext,
        origin: MessageOrigin,
        cmd: Self::Cmd,
    ) -> TransactionResult {
        let status = match cmd {
            Command::SetConfiguration { skip_stat } => {
                let o = origin.account()?;
                log::info!("SetConfiguration: [{}] -> {}", hex::encode(&o), skip_stat);

                if skip_stat {
                    self.no_tracking.insert(o, skip_stat);
                } else {
                    self.no_tracking.remove(&o);
                }

                Ok(())
            }
        };

        status
    }

    fn handle_query(&mut self, origin: Option<&chain::AccountId>, req: Request) -> Response {
        let inner = || -> Result<Response> {
            match req {
                Request::SetPageView {
                    page_views,
                    encrypted,
                } => {
                    for page_view in page_views {
                        if page_view.uid.len() == 64
                            && self
                                .no_tracking
                                .contains_key(&account_id_from_hex(&page_view.uid)?)
                        {
                            continue;
                        }
                        let b = self
                            .page_views
                            .clone()
                            .into_iter()
                            .any(|x| x.id == page_view.id);
                        if !b {
                            self.page_views.push(page_view);
                        }
                    }

                    self.encrypted = encrypted;

                    Ok(Response::SetPageView {
                        page_view_count: self.page_views.len() as u32,
                    })
                }
                Request::ClearPageView { timestamp: _ } => {
                    self.page_views.clear();
                    Ok(Response::ClearPageView {
                        page_view_count: self.page_views.len() as u32,
                    })
                }
                Request::GetOnlineUsers { start, end } => {
                    self.update_online_users(start, end);
                    Ok(Response::GetOnlineUsers {
                        online_users: self.online_users.clone(),
                        encrypted: self.encrypted,
                    })
                }
                Request::GetHourlyStats {
                    start,
                    end,
                    start_of_week,
                } => {
                    self.update_hourly_stats(start, end, start_of_week);
                    Ok(Response::GetHourlyStats {
                        hourly_stat: self.hourly_stat.clone(),
                        encrypted: self.encrypted,
                    })
                }
                Request::GetDailyStats { daily_stat } => {
                    self.update_daily_stats(daily_stat);
                    Ok(Response::GetDailyStats {
                        daily_stat: self.daily_stat.clone(),
                        encrypted: self.encrypted,
                    })
                }
                Request::GetWeeklySites {
                    weekly_sites_in_db,
                    weekly_sites_new,
                } => {
                    self.update_weekly_sites(weekly_sites_in_db, weekly_sites_new);
                    Ok(Response::GetWeeklySites {
                        weekly_sites: self.weekly_sites.clone(),
                        encrypted: self.encrypted,
                    })
                }
                Request::GetWeeklyDevices {
                    weekly_devices_in_db,
                    weekly_devices_new,
                } => {
                    self.update_weekly_devices(weekly_devices_in_db, weekly_devices_new);
                    Ok(Response::GetWeeklyDevices {
                        weekly_devices: self.weekly_devices.clone(),
                        encrypted: self.encrypted,
                    })
                }
                Request::GetTotalStat { total_stat, count } => {
                    self.update_total_stat(total_stat, count);
                    Ok(Response::GetTotalStat {
                        total_stat: self.total_stat.clone(),
                        encrypted: self.encrypted,
                    })
                }
                Request::GetConfiguration { account } => {
                    if origin == None || origin.unwrap() != &account {
                        return Err(anyhow::Error::msg(Error::NotAuthorized));
                    }

                    let mut off = false;
                    if let Some(o) = self.no_tracking.get(&account) {
                        off = *o;
                    }
                    Ok(Response::GetConfiguration { skip_stat: off })
                }
            }
        };
        match inner() {
            Err(error) => Response::Error(error.to_string()),
            Ok(resp) => resp,
        }
    }
}
