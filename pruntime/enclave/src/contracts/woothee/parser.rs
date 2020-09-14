use crate::std::borrow::Cow;

use regex::Regex;

use super::dataset;
use super::woothee::VALUE_UNKNOWN;

#[macro_use]
use lazy_static;

lazy_static! {
    static ref RX_CHROME_PATTERN: Regex = Regex::new(r"(?:Chrome|CrMo|CriOS)/([.0-9]+)").unwrap();
    static ref RX_DOCOMO_VERSION_PATTERN: Regex = Regex::new(r#"DoCoMo/[.0-9]+[ /]([^- /;()"']+)"#).unwrap();
    static ref RX_VIVALDI_PATTERN: Regex = Regex::new(r"Vivaldi/([.0-9]+)").unwrap();
    static ref RX_FIREFOX_PATTERN: Regex = Regex::new(r"Firefox/([.0-9]+)").unwrap();
    static ref RX_FIREFOX_OS_PATTERN: Regex =
        Regex::new(r"^Mozilla/[.0-9]+ \((?:Mobile|Tablet);(?:.*;)? rv:([.0-9]+)\) Gecko/[.0-9]+ Firefox/[.0-9]+$")
            .unwrap();
    static ref RX_FIREFOX_IOS_PATTERN: Regex = Regex::new(r"FxiOS/([.0-9]+)").unwrap();
    static ref RX_FOMA_VERSION_PATTERN: Regex = Regex::new(r"\(([^;)]+);FOMA;").unwrap();
    static ref RX_JIG_PATTERN: Regex = Regex::new(r"jig browser[^;]+; ([^);]+)").unwrap();
    static ref RX_KDDI_PATTERN: Regex = Regex::new(r#"KDDI-([^- /;()"']+)"#).unwrap();
    static ref RX_MAYBE_RSS_PATTERN: Regex = Regex::new(r"(?i)rss(?:reader|bar|[-_ /;()]|[ +]*/)").unwrap();
    static ref RX_MAYBE_CRAWLER_PATTERN: Regex = Regex::new(r"(?i)(?:bot|crawler|spider)(?:[-_ ./;@()]|$)").unwrap();
    static ref RX_MAYBE_FEED_PARSER_PATTERN: Regex = Regex::new(r"(?i)(?:feed|web) ?parser").unwrap();
    static ref RX_MAYBE_WATCHDOG_PATTERN: Regex = Regex::new(r"(?i)watch ?dog").unwrap();
    static ref RX_MSEDGE_PATTERN: Regex = Regex::new(r"(?:Edge|Edg|EdgiOS|EdgA)/([.0-9]+)").unwrap();
    static ref RX_MSIE_PATTERN: Regex = Regex::new(r"MSIE ([.0-9]+);").unwrap();
    static ref RX_OPERA_VERSION_PATTERN1: Regex = Regex::new(r"Version/([.0-9]+)").unwrap();
    static ref RX_OPERA_VERSION_PATTERN2: Regex = Regex::new(r"Opera[/ ]([.0-9]+)").unwrap();
    static ref RX_OPERA_VERSION_PATTERN3: Regex = Regex::new(r"OPR/([.0-9]+)").unwrap();
    static ref RX_GSA_VERSION_PATTERN: Regex = Regex::new(r"GSA/([.0-9]+)").unwrap();
    static ref RX_SAFARI_PATTERN: Regex = Regex::new(r"Version/([.0-9]+)").unwrap();
    static ref RX_SOFTBANK_PATTERN: Regex = Regex::new(r"(?:SoftBank|Vodafone|J-PHONE)/[.0-9]+/([^ /;()]+)").unwrap();
    static ref RX_TRIDENT_PATTERN: Regex = Regex::new(r"Trident/([.0-9]+);").unwrap();
    static ref RX_TRIDENT_VERSION_PATTERN: Regex = Regex::new(r" rv:([.0-9]+)").unwrap();
    static ref RX_IEMOBILE_PATTERN: Regex = Regex::new(r"IEMobile/([.0-9]+);").unwrap();
    static ref RX_WILLCOM_PATTERN: Regex = Regex::new(r"(?:WILLCOM|DDIPOCKET);[^/]+/([^ /;()]+)").unwrap();
    static ref RX_WINDOWS_VERSION_PATTERN: Regex = Regex::new(r"Windows ([ .a-zA-Z0-9]+)[;\\)]").unwrap();
    static ref RX_WIN_PHONE: Regex = Regex::new(r"^Phone(?: OS)? ([.0-9]+)").unwrap();
    static ref RX_WEBVIEW_PATTERN: Regex = Regex::new(r"iP(hone;|ad;|od) .*like Mac OS X").unwrap();
    static ref RX_WEBVIEW_VERSION_PATTERN: Regex = Regex::new(r"Version/([.0-9]+)").unwrap();
    static ref RX_PPC_OS_VERSION: Regex = Regex::new(r"rv:(\d+\.\d+\.\d+)").unwrap();
    static ref RX_FREEBSD_OS_VERSION: Regex = Regex::new(r"FreeBSD ([^;\)]+);").unwrap();
    static ref RX_CHROMEOS_OS_VERSION: Regex = Regex::new(r"CrOS ([^\)]+)\)").unwrap();
    static ref RX_ANDROIDOS_OS_VERSION: Regex = Regex::new(r"Android[- ](\d+(?:\.\d+(?:\.\d+)?)?)").unwrap();
    static ref RX_PSP_OS_VERSION: Regex = Regex::new(r"PSP \(PlayStation Portable\); ([.0-9]+)\)").unwrap();
    static ref RX_PS3_OS_VERSION: Regex = Regex::new(r"PLAYSTATION 3;? ([.0-9]+)\)").unwrap();
    static ref RX_PSVITA_OS_VERSION: Regex = Regex::new(r"PlayStation Vita ([.0-9]+)\)").unwrap();
    static ref RX_PS4_OS_VERSION: Regex = Regex::new(r"PlayStation 4 ([.0-9]+)\)").unwrap();
    static ref RX_BLACKBERRY10_OS_VERSION: Regex = Regex::new(r"BB10(?:.+)Version/([.0-9]+) ").unwrap();
    static ref RX_BLACKBERRY_OS_VERSION: Regex = Regex::new(r"BlackBerry(?:\d+)/([.0-9]+) ").unwrap();
    static ref RE_OSX_IPHONE_OS_VERSION: Regex =
        Regex::new(r"; CPU(?: iPhone)? OS (\d+_\d+(?:_\d+)?) like Mac OS X").unwrap();
    static ref RE_OSX_OS_VERSION: Regex = Regex::new(r"Mac OS X (10[._]\d+(?:[._]\d+)?)(?:\)|;)").unwrap();
    static ref RX_HTTP_CLIENT: Regex =
        Regex::new(r"^(?:Apache-HttpClient/|Jakarta Commons-HttpClient/|Java/)").unwrap();
    static ref RX_HTTP_CLIENT_OTHER: Regex = Regex::new(r"[- ]HttpClient(/|$)").unwrap();
    static ref RX_PHP: Regex = Regex::new(r"^(?:PHP|WordPress|CakePHP|PukiWiki|PECL::HTTP)(?:/| |$)").unwrap();
    static ref RX_PEAR: Regex = Regex::new(r"(?:PEAR HTTP_Request|HTTP_Request)(?: class|2)").unwrap();
    static ref RX_MAYBE_CRAWLER_OTHER: Regex =
        Regex::new(r"(?:Rome Client |UnwindFetchor/|ia_archiver |Summify |PostRank/)").unwrap();
    static ref RE_SLEIPNIR_VERSION: Regex = Regex::new(r"Sleipnir/([.0-9]+)").unwrap();
    static ref RX_YABROWSER_VERSION: Regex = Regex::new(r"YaBrowser/(\d+\.\d+\.\d+\.\d+)").unwrap();
}

#[derive(Debug, Default)]
pub struct WootheeResult<'a> {
    pub name: &'a str,
    pub category: &'a str,
    pub os: &'a str,
    pub os_version: Cow<'a, str>,
    //pub os_version: &'a str,
    pub browser_type: &'a str,
    pub version: &'a str,
    pub vendor: &'a str,
}

impl<'a> WootheeResult<'a> {
    pub fn new() -> WootheeResult<'a> {
        WootheeResult {
            name: VALUE_UNKNOWN,
            category: VALUE_UNKNOWN,
            os: VALUE_UNKNOWN,
            os_version: VALUE_UNKNOWN.into(),
            //os_version: VALUE_UNKNOWN,
            browser_type: VALUE_UNKNOWN,
            version: VALUE_UNKNOWN,
            vendor: VALUE_UNKNOWN,
        }
    }

    fn populate_with(&mut self, ds: &WootheeResult<'a>) {
        if !ds.name.is_empty() {
            self.name = ds.name;
        }

        if !ds.category.is_empty() {
            self.category = ds.category;
        }

        if !ds.os.is_empty() {
            self.os = ds.os;
        }

        if !ds.browser_type.is_empty() {
            self.browser_type = ds.browser_type;
        }

        if !ds.version.is_empty() {
            self.version = ds.version;
        }

        if !ds.vendor.is_empty() {
            self.vendor = ds.vendor;
        }
    }
}

#[derive(Default, Debug)]
pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    pub fn parse<'a>(&self, agent: &'a str) -> Option<WootheeResult<'a>> {
        let mut result = WootheeResult::new();
        if agent == "" || agent == "-" {
            return Some(result);
        }

        if self.try_crawler(agent, &mut result) {
            return Some(result);
        }

        if self.try_browser(agent, &mut result) {
            self.try_os(agent, &mut result);
            return Some(result);
        }

        if self.try_mobilephone(agent, &mut result) {
            return Some(result);
        }

        if self.try_appliance(agent, &mut result) {
            return Some(result);
        }

        if self.try_misc(agent, &mut result) {
            return Some(result);
        }

        if self.try_os(agent, &mut result) {
            return Some(result);
        }

        if self.try_rare_cases(agent, &mut result) {
            return Some(result);
        }

        None
    }

    fn populate_dataset(&self, result: &mut WootheeResult, label: &str) -> bool {
        match self.lookup_dataset(label) {
            Some(ds) => {
                result.populate_with(ds);
                true
            }
            None => false,
        }
    }

    fn lookup_dataset(&self, label: &str) -> Option<&WootheeResult<'static>> {
        dataset::DATASET.get(label)
    }

    pub fn try_crawler(&self, agent: &str, result: &mut WootheeResult) -> bool {
        if self.challenge_google(agent, result) {
            return true;
        }

        if self.challenge_crawlers(agent, result) {
            return true;
        }

        false
    }

    fn try_os<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if self.challenge_windows(agent, result) {
            return true;
        }

        if self.challenge_osx(agent, result) {
            return true;
        }

        if self.challenge_linux(agent, result) {
            return true;
        }

        if self.challenge_smartphone(agent, result) {
            return true;
        }

        if self.challenge_mobilephone(agent, result) {
            return true;
        }

        if self.challenge_appliance(agent, result) {
            return true;
        }

        if self.challenge_misc_os(agent, result) {
            return true;
        }

        false
    }

    fn try_browser<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if self.challenge_msie(agent, result) {
            return true;
        }

        if self.challenge_ms_edge(agent, result) {
            return true;
        }

        if self.challenge_vivaldi(agent, result) {
            return true;
        }

        if self.challenge_firefox_ios(agent, result) {
            return true;
        }

        if self.challenge_yandexbrowser(agent, result) {
            return true;
        }

        if self.challenge_safari_chrome(agent, result) {
            return true;
        }

        if self.challenge_firefox(agent, result) {
            return true;
        }

        if self.challenge_opera(agent, result) {
            return true;
        }

        if self.challenge_webview(agent, result) {
            return true;
        }

        false
    }

    fn try_mobilephone<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if self.challenge_docomo(agent, result) {
            return true;
        }

        if self.challenge_au(agent, result) {
            return true;
        }

        if self.challenge_softbank(agent, result) {
            return true;
        }

        if self.challenge_willcom(agent, result) {
            return true;
        }

        if self.challenge_misc_mobilephone(agent, result) {
            return true;
        }

        false
    }

    fn try_appliance<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if self.challenge_playstation(agent, result) {
            return true;
        }

        if self.challenge_nintendo(agent, result) {
            return true;
        }

        if self.challenge_digital_tv(agent, result) {
            return true;
        }

        false
    }
    fn try_misc(&self, agent: &str, result: &mut WootheeResult) -> bool {
        if self.challenge_desktop_tools(agent, result) {
            return true;
        }

        false
    }

    fn try_rare_cases<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if self.challenge_smartphone_patterns(agent, result) {
            return true;
        }

        if self.challenge_sleipnir(agent, result) {
            return true;
        }

        if self.challenge_http_library(agent, result) {
            return true;
        }

        if self.challenge_maybe_rss_reader(agent, result) {
            return true;
        }

        if self.challenge_maybe_crawler(agent, result) {
            return true;
        }

        false
    }

    fn challenge_google(&self, agent: &str, result: &mut WootheeResult) -> bool {
        if !agent.contains("Google") {
            return false;
        }

        if agent.contains("compatible; Googlebot") {
            if agent.contains("compatible; Googlebot-Mobile") {
                return self.populate_dataset(result, "GoogleBotMobile");
            }
            return self.populate_dataset(result, "GoogleBot");
        }

        if agent.contains("Googlebot-Image/") {
            return self.populate_dataset(result, "GoogleBot");
        }

        if agent.contains("Mediapartners-Google")
            && (agent.contains("compatible; Mediapartners-Google") || agent == "Mediapartners-Google")
        {
            return self.populate_dataset(result, "GoogleMediaPartners");
        }

        if agent.contains("Feedfetcher-Google;") {
            return self.populate_dataset(result, "GoogleFeedFetcher");
        }

        if agent.contains("AppEngine-Google") {
            return self.populate_dataset(result, "GoogleAppEngine");
        }

        if agent.contains("Google Web Preview") {
            return self.populate_dataset(result, "GoogleWebPreview");
        }

        false
    }

    fn challenge_crawlers(&self, agent: &str, result: &mut WootheeResult) -> bool {
        if agent.contains("Yahoo")
            || agent.contains("help.yahoo.co.jp/help/jp/")
            || agent.contains("listing.yahoo.co.jp/support/faq/")
        {
            if agent.contains("compatible; Yahoo! Slurp") {
                return self.populate_dataset(result, "YahooSlurp");
            } else if agent.contains("YahooFeedSeekerJp")
                || agent.contains("YahooFeedSeekerBetaJp")
                || agent.contains("crawler (http://listing.yahoo.co.jp/support/faq/")
                || agent.contains("crawler (http://help.yahoo.co.jp/help/jp/")
                || agent.contains("Y!J-BRZ/YATSHA crawler")
                || agent.contains("Y!J-BRY/YATSH crawler")
            {
                return self.populate_dataset(result, "YahooJP");
            } else if agent.contains("Yahoo Pipes") {
                return self.populate_dataset(result, "YahooPipes");
            }
        }

        if agent.contains("msnbot") {
            return self.populate_dataset(result, "msnbot");
        }

        if agent.contains("bingbot") && agent.contains("compatible; bingbot") {
            return self.populate_dataset(result, "bingbot");
        }

        if agent.contains("BingPreview") {
            return self.populate_dataset(result, "BingPreview");
        }

        if agent.contains("Baidu")
            && (agent.contains("compatible; Baiduspider")
                || agent.contains("Baiduspider+")
                || agent.contains("Baiduspider-image+"))
        {
            return self.populate_dataset(result, "Baiduspider");
        }

        if agent.contains("Yeti")
            && (agent.contains("http://help.naver.com/robots")
                || agent.contains("http://help.naver.com/support/robots.html")
                || agent.contains("http://naver.me/bot"))
        {
            return self.populate_dataset(result, "Yeti");
        }

        if agent.contains("FeedBurner/") {
            return self.populate_dataset(result, "FeedBurner");
        }

        if agent.contains("facebookexternalhit") {
            return self.populate_dataset(result, "facebook");
        }

        if agent.contains("Twitterbot/") {
            return self.populate_dataset(result, "twitter");
        }

        if agent.contains("ichiro")
            && (agent.contains("http://help.goo.ne.jp/door/crawler.html")
                || agent.contains("compatible; ichiro/mobile goo;"))
        {
            return self.populate_dataset(result, "goo");
        }

        if agent.contains("gooblogsearch/") {
            return self.populate_dataset(result, "goo");
        }

        if agent.contains("Apple-PubSub") {
            return self.populate_dataset(result, "ApplePubSub");
        }

        if agent.contains("(www.radian6.com/crawler)") {
            return self.populate_dataset(result, "radian6");
        }

        if agent.contains("Genieo/") {
            return self.populate_dataset(result, "Genieo");
        }

        if agent.contains("labs.topsy.com/butterfly/") {
            return self.populate_dataset(result, "topsyButterfly");
        }

        if agent.contains("rogerbot/1.0 (http://www.seomoz.org/dp/rogerbot") {
            return self.populate_dataset(result, "rogerbot");
        }

        if agent.contains("compatible; AhrefsBot/") {
            return self.populate_dataset(result, "AhrefsBot");
        }

        if agent.contains("livedoor FeedFetcher") || agent.contains("Fastladder FeedFetcher") {
            return self.populate_dataset(result, "livedoorFeedFetcher");
        }

        if agent.contains("Hatena Antenna")
            || agent.contains("Hatena Pagetitle Agent")
            || agent.contains("Hatena Diary RSS")
        {
            return self.populate_dataset(result, "Hatena");
        }

        if agent.contains("mixi-check") || agent.contains("mixi-crawler") || agent.contains("mixi-news-crawler") {
            return self.populate_dataset(result, "mixi");
        }

        if agent.contains("Indy Library") && agent.contains("compatible; Indy Library") {
            return self.populate_dataset(result, "IndyLibrary");
        }

        if agent.contains("trendictionbot") {
            return self.populate_dataset(result, "trendictionbot");
        }

        false
    }

    fn challenge_msie<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if !agent.contains("compatible; MSIE") && !agent.contains("Trident/") && !agent.contains("IEMobile") {
            return false;
        }

        let re_msie_caps = RX_MSIE_PATTERN.captures(agent);
        let re_trident_caps = RX_TRIDENT_PATTERN.captures(agent);
        let re_trident_ver_caps = RX_TRIDENT_VERSION_PATTERN.captures(agent);
        let re_ie_mobile_caps = RX_IEMOBILE_PATTERN.captures(agent);

        result.version = if let Some(c) = re_msie_caps {
            c.get(1).unwrap().as_str()
        } else if re_trident_caps.is_some() {
            if let Some(caps) = re_trident_ver_caps {
                caps.get(1).unwrap().as_str()
            } else {
                VALUE_UNKNOWN
            }
        } else if re_ie_mobile_caps.is_some() {
            re_ie_mobile_caps.unwrap().get(1).unwrap().as_str()
        } else {
            VALUE_UNKNOWN
        };

        if !self.populate_dataset(result, "MSIE") {
            return false;
        }

        true
    }

    fn challenge_ms_edge<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        match RX_MSEDGE_PATTERN.captures(agent) {
            Some(caps) => {
                result.version = caps.get(1).unwrap().as_str();
            }
            None => {
                return false;
            }
        };

        if !self.populate_dataset(result, "Edge") {
            return false;
        }

        true
    }

    fn challenge_vivaldi<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        match RX_VIVALDI_PATTERN.captures(agent) {
            Some(caps) => {
                result.version = caps.get(1).unwrap().as_str();
            }
            None => {
                return false;
            }
        };

        if !self.populate_dataset(result, "Vivaldi") {
            return false;
        }

        true
    }

    fn challenge_firefox_ios<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        match RX_FIREFOX_IOS_PATTERN.captures(agent) {
            Some(caps) => {
                result.version = caps.get(1).unwrap().as_str();
            }
            None => {
                return false;
            }
        };

        if !self.populate_dataset(result, "Firefox") {
            return false;
        }

        true
    }

    fn challenge_yandexbrowser<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if !agent.contains("YaBrowser/") {
            return false;
        }

        match RX_YABROWSER_VERSION.captures(agent) {
            Some(caps) => {
                result.version = caps.get(1).unwrap().as_str();
            }
            None => {
                return false;
            }
        }

        if !self.populate_dataset(result, "YaBrowser") {
            return false;
        }

        true
    }

    fn challenge_safari_chrome<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if agent.contains("Chrome") && agent.contains("wv") {
            return false;
        }

        if !agent.contains("Safari/") {
            return false;
        }

        if RX_CHROME_PATTERN.is_match(agent) {
            if RX_OPERA_VERSION_PATTERN3.is_match(agent) {
                let version = match RX_OPERA_VERSION_PATTERN3.captures(agent) {
                    Some(caps) => caps.get(1).unwrap().as_str(),
                    None => "",
                };
                if !self.populate_dataset(result, "Opera") {
                    return false;
                }
                if !version.is_empty() {
                    result.version = version;
                }
                return true;
            }

            if !self.populate_dataset(result, "Chrome") {
                return false;
            }

            let version = match RX_CHROME_PATTERN.captures(agent) {
                Some(caps) => caps.get(1).unwrap().as_str(),
                None => "",
            };
            if !version.is_empty() {
                result.version = version;
            }
            return true;
        }

        let version = match RX_SAFARI_PATTERN.captures(agent) {
            Some(caps) => caps.get(1).unwrap().as_str(),
            None => VALUE_UNKNOWN,
        };

        if agent.contains("GSA") {
            if let Some(caps) = RX_GSA_VERSION_PATTERN.captures(agent) {
                result.version = caps.get(1).unwrap().as_str();
                if !self.populate_dataset(result, "GSA") {
                    return false;
                }
                return true;
            }
        }

        if !self.populate_dataset(result, "Safari") {
            return false;
        }

        result.version = version;

        true
    }

    fn challenge_firefox<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if !agent.contains("Firefox/") {
            return false;
        }

        let version = match RX_FIREFOX_PATTERN.captures(agent) {
            Some(caps) => caps.get(1).unwrap().as_str(),
            None => VALUE_UNKNOWN,
        };

        if !self.populate_dataset(result, "Firefox") {
            return false;
        }

        result.version = version;

        true
    }

    fn challenge_opera<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if !agent.contains("Opera") {
            return false;
        }

        let version = match RX_OPERA_VERSION_PATTERN1.captures(agent) {
            Some(caps) => caps.get(1).unwrap().as_str(),
            None => match RX_OPERA_VERSION_PATTERN2.captures(agent) {
                Some(caps2) => caps2.get(1).unwrap().as_str(),
                None => VALUE_UNKNOWN,
            },
        };

        if !self.populate_dataset(result, "Opera") {
            return false;
        }

        result.version = version;

        true
    }

    fn challenge_webview<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        let version = if RX_WEBVIEW_VERSION_PATTERN.is_match(agent) {
            match RX_WEBVIEW_VERSION_PATTERN.captures(agent) {
                Some(caps) => caps.get(1).unwrap().as_str(),
                None => "",
            }
        } else {
            VALUE_UNKNOWN
        };

        if agent.contains("Chrome") && agent.contains("wv") {
            result.version = version;
            self.populate_dataset(result, "Webview");
            return true;
        }

        if !RX_WEBVIEW_PATTERN.is_match(agent) || agent.contains("Safari/") {
            return false;
        }

        if !self.populate_dataset(result, "Webview") {
            return false;
        }

        let version = match RX_WEBVIEW_VERSION_PATTERN.captures(agent) {
            Some(caps) => caps.get(1).unwrap().as_str(),
            None => "",
        };
        if !version.is_empty() {
            result.version = version;
        }

        true
    }

    fn challenge_docomo<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if !agent.contains("DoCoMo") && !agent.contains(";FOMA;") {
            return false;
        }

        let mut version = VALUE_UNKNOWN;
        let docomo_caps = RX_DOCOMO_VERSION_PATTERN.captures(agent);
        let foma_caps = RX_FOMA_VERSION_PATTERN.captures(agent);
        if let Some(c) = docomo_caps {
            version = c.get(1).unwrap().as_str();
        } else if let Some(c) = foma_caps {
            version = c.get(1).unwrap().as_str();
        }

        if !self.populate_dataset(result, "docomo") {
            return false;
        }
        result.version = version;

        true
    }

    fn challenge_au<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if !agent.contains("KDDI-") {
            return false;
        }

        let mut version = VALUE_UNKNOWN;
        let caps = RX_KDDI_PATTERN.captures(agent);
        if let Some(c) = caps {
            version = c.get(1).unwrap().as_str();
        }

        if !self.populate_dataset(result, "au") {
            return false;
        }
        result.version = version;

        true
    }

    fn challenge_softbank<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if !agent.contains("SoftBank") && !agent.contains("Vodafone") && !agent.contains("J-PHONE") {
            return false;
        }

        let mut version = VALUE_UNKNOWN;
        let caps = RX_SOFTBANK_PATTERN.captures(agent);
        if let Some(c) = caps {
            version = c.get(1).unwrap().as_str();
        }

        if !self.populate_dataset(result, "SoftBank") {
            return false;
        }
        result.version = version;

        true
    }

    fn challenge_willcom<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if !agent.contains("WILLCOM") && !agent.contains("DDIPOCKET") {
            return false;
        }

        let mut version = VALUE_UNKNOWN;
        let caps = RX_WILLCOM_PATTERN.captures(agent);
        if let Some(c) = caps {
            version = c.get(1).unwrap().as_str();
        }

        if !self.populate_dataset(result, "willcom") {
            return false;
        }
        result.version = version;

        true
    }

    fn challenge_misc_mobilephone<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if agent.contains("jig browser") {
            if !self.populate_dataset(result, "jig") {
                return false;
            }

            let caps = RX_JIG_PATTERN.captures(agent);
            if let Some(c) = caps {
                result.version = c.get(1).unwrap().as_str();
            }
            return true;
        }

        if agent.contains("emobile/") || agent.contains("OpenBrowser") || agent.contains("Browser/Obigo-Browser") {
            if !self.populate_dataset(result, "emobile") {
                return false;
            }
            return true;
        }

        if agent.contains("SymbianOS") {
            if !self.populate_dataset(result, "SymbianOS") {
                return false;
            }
            return true;
        }

        if agent.contains("Hatena-Mobile-Gateway/") {
            if !self.populate_dataset(result, "MobileTranscoder") {
                return false;
            }
            result.version = "Hatena";
            return true;
        }

        if agent.contains("livedoor-Mobile-Gateway/") {
            if !self.populate_dataset(result, "MobileTranscoder") {
                return false;
            }
            result.version = "livedoor";
            return true;
        }

        false
    }

    fn challenge_playstation<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        let mut os_version = "";

        let d = if agent.contains("PSP (PlayStation Portable)") {
            os_version = match RX_PSP_OS_VERSION.captures(agent) {
                Some(caps) => caps.get(1).unwrap().as_str(),
                None => "",
            };
            self.lookup_dataset("PSP")
        } else if agent.contains("PlayStation Vita") {
            os_version = match RX_PSVITA_OS_VERSION.captures(agent) {
                Some(caps) => caps.get(1).unwrap().as_str(),
                None => "",
            };
            self.lookup_dataset("PSVita")
        } else if agent.contains("PLAYSTATION 3 ") || agent.contains("PLAYSTATION 3;") {
            os_version = match RX_PS3_OS_VERSION.captures(agent) {
                Some(caps) => caps.get(1).unwrap().as_str(),
                None => "",
            };
            self.lookup_dataset("PS3")
        } else if agent.contains("PlayStation 4 ") {
            os_version = match RX_PS4_OS_VERSION.captures(agent) {
                Some(caps) => caps.get(1).unwrap().as_str(),
                None => "",
            };
            self.lookup_dataset("PS4")
        } else {
            None
        };

        if d.is_none() {
            return false;
        }
        let data = d.unwrap();

        result.populate_with(data);

        if !os_version.is_empty() {
            result.os_version = os_version.into();
        }

        true
    }

    fn challenge_nintendo(&self, agent: &str, result: &mut WootheeResult) -> bool {
        if agent.contains("Nintendo 3DS;") {
            return self.populate_dataset(result, "Nintendo3DS");
        }

        if agent.contains("Nintendo DSi;") {
            return self.populate_dataset(result, "NintendoDSi");
        }

        if agent.contains("Nintendo Wii;") {
            return self.populate_dataset(result, "NintendoWii");
        }

        if agent.contains("(Nintendo WiiU)") {
            return self.populate_dataset(result, "NintendoWiiU");
        }

        false
    }

    fn challenge_digital_tv(&self, agent: &str, result: &mut WootheeResult) -> bool {
        if agent.contains("InettvBrowser/") {
            return self.populate_dataset(result, "DigitalTV");
        }

        false
    }

    fn challenge_windows<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if !agent.contains("Windows") {
            return false;
        }

        if agent.contains("Xbox") {
            if agent.contains("Xbox; Xbox One)") {
                return self.populate_dataset(result, "XboxOne");
            }
            return self.populate_dataset(result, "Xbox360");
        }

        let mut w = self.lookup_dataset("Win");
        if w.is_none() {
            return false;
        }
        let mut win = w.unwrap();

        let caps = RX_WINDOWS_VERSION_PATTERN.captures(agent);
        if caps.is_none() {
            result.category = win.category;
            result.os = win.name;
            return true;
        }

        let mut version = caps.unwrap().get(1).unwrap().as_str();
        w = match version {
            "NT 10.0" => self.lookup_dataset("Win10"),
            "NT 6.3" => self.lookup_dataset("Win8.1"),
            "NT 6.2" => self.lookup_dataset("Win8"),
            "NT 6.1" => self.lookup_dataset("Win7"),
            "NT 6.0" => self.lookup_dataset("WinVista"),
            "NT 5.1" => self.lookup_dataset("WinXP"),
            "NT 5.0" => self.lookup_dataset("Win2000"),
            "NT 4.0" => self.lookup_dataset("WinNT4"),
            "98" => self.lookup_dataset("Win98"),
            "95" => self.lookup_dataset("Win95"),
            "CE" => self.lookup_dataset("WinCE"),
            _ => {
                let caps = RX_WIN_PHONE.captures(version);
                if let Some(c) = caps {
                    version = c.get(1).unwrap().as_str();
                    self.lookup_dataset("WinPhone")
                } else {
                    None
                }
            }
        };

        if w.is_none() {
            return false;
        }
        win = w.unwrap();

        result.category = win.category;
        result.os = win.name;
        if !version.is_empty() {
            result.os_version = version.into();
        }

        true
    }

    fn challenge_osx<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if !agent.contains("Mac OS X") {
            return false;
        }

        let mut d = self.lookup_dataset("OSX");
        if d.is_none() {
            return false;
        }
        let mut data = d.unwrap();
        //let mut version: Cow<'a, str> = "".into();
        let mut version = "";

        if agent.contains("like Mac OS X") {
            if agent.contains("iPhone;") {
                d = self.lookup_dataset("iPhone");
            } else if agent.contains("iPad;") {
                d = self.lookup_dataset("iPad");
            } else if agent.contains("iPod") {
                d = self.lookup_dataset("iPod");
            }
            if d.is_none() {
                return false;
            }
            data = d.unwrap();

            let caps = RE_OSX_IPHONE_OS_VERSION.captures(agent);
            if let Some(c) = caps {
                //let v = c.get(1).unwrap().as_str();
                //version = v.replace("_", ".").into();
                version = c.get(1).unwrap().as_str();
            }
        } else {
            let caps = RE_OSX_OS_VERSION.captures(agent);
            if let Some(c) = caps {
                //let v = c.get(1).unwrap().as_str();
                //version = v.replace("_", ".");//.into();
                version = c.get(1).unwrap().as_str();
            }
        }

        result.category = data.category;
        result.os = data.name;
        if !version.is_empty() {
            result.os_version = version.into();
        }

        true
    }

    fn challenge_linux<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if !agent.contains("Linux") {
            return false;
        }

        let mut os_version = "";
        let d = if agent.contains("Android") {
            let caps = RX_ANDROIDOS_OS_VERSION.captures(agent);
            if let Some(c) = caps {
                os_version = c.get(1).unwrap().as_str();
            }
            self.lookup_dataset("Android")
        } else {
            self.lookup_dataset("Linux")
        };

        if d.is_none() {
            return false;
        }
        let data = d.unwrap();

        result.category = data.category;
        result.os = data.name;
        if !os_version.is_empty() {
            result.os_version = os_version.into();
        }

        true
    }

    fn challenge_smartphone<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        let mut os_version = "";

        let mut d = if agent.contains("iPhone") {
            self.lookup_dataset("iPhone")
        } else if agent.contains("iPad") {
            self.lookup_dataset("iPad")
        } else if agent.contains("iPod") {
            self.lookup_dataset("iPod")
        } else if agent.contains("Android") {
            self.lookup_dataset("Android")
        } else if agent.contains("CFNetwork") {
            self.lookup_dataset("iOS")
        } else if agent.contains("BB10") {
            let caps = RX_BLACKBERRY10_OS_VERSION.captures(agent);
            if let Some(c) = caps {
                os_version = c.get(1).unwrap().as_str();
            }
            result.version = VALUE_UNKNOWN;
            self.lookup_dataset("BlackBerry10")
        } else if agent.contains("BlackBerry") {
            let caps = RX_BLACKBERRY_OS_VERSION.captures(agent);
            if let Some(c) = caps {
                os_version = c.get(1).unwrap().as_str();
            }
            self.lookup_dataset("BlackBerry")
        } else {
            None
        };

        let f = self.lookup_dataset("Firefox");
        if f.is_none() {
            return false;
        }
        let firefox = f.unwrap();

        if result.name == firefox.name {
            // Firefox OS specific pattern
            // http://lawrencemandel.com/2012/07/27/decision-made-firefox-os-user-agent-string/
            // https://github.com/woothee/woothee/issues/2
            let caps = RX_FIREFOX_OS_PATTERN.captures(agent);
            if let Some(c) = caps {
                if c.len() > 1 {
                    d = self.lookup_dataset("FirefoxOS");
                    os_version = c.get(1).unwrap().as_str();
                }
            }
        }

        if d.is_none() {
            return false;
        }
        let data = d.unwrap();

        result.category = data.category;
        result.os = data.name;
        if !os_version.is_empty() {
            result.os_version = os_version.into();
        }

        true
    }

    fn challenge_mobilephone<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if agent.contains("KDDI-") {
            let caps = RX_KDDI_PATTERN.captures(agent);
            if let Some(c) = caps {
                let term = c.get(1).unwrap().as_str();
                let d = self.lookup_dataset("au");
                if d.is_none() {
                    return false;
                }
                let data = d.unwrap();
                result.category = data.category;
                result.os = data.os;
                result.version = term;
                return true;
            }
        }

        if agent.contains("WILLCOM") || agent.contains("DDIPOCKET") {
            let caps = RX_WILLCOM_PATTERN.captures(agent);
            if let Some(c) = caps {
                let term = c.get(1).unwrap().as_str();
                let d = self.lookup_dataset("willcom");
                if d.is_none() {
                    return false;
                }
                let data = d.unwrap();
                result.category = data.category;
                result.os = data.os;
                result.version = term;
                return true;
            }
        }

        if agent.contains("SymbianOS") {
            let d = self.lookup_dataset("SymbianOS");
            if d.is_none() {
                return false;
            }
            let data = d.unwrap();
            result.category = data.category;
            result.os = data.os;
            return true;
        }

        if agent.contains("Google Wireless Transcoder") {
            if !self.populate_dataset(result, "MobileTranscoder") {
                return false;
            }
            result.version = "Google";
        }

        if agent.contains("Naver Transcoder") {
            if !self.populate_dataset(result, "MobileTranscoder") {
                return false;
            }
            result.version = "Naver";
        }

        false
    }

    fn challenge_desktop_tools(&self, agent: &str, result: &mut WootheeResult) -> bool {
        if agent.contains("AppleSyndication/") {
            return self.populate_dataset(result, "SafariRSSReader");
        }

        if agent.contains("compatible; Google Desktop/") {
            return self.populate_dataset(result, "GoogleDesktop");
        }

        if agent.contains("Windows-RSS-Platform") {
            return self.populate_dataset(result, "WindowsRSSReader");
        }

        false
    }

    fn challenge_smartphone_patterns(&self, agent: &str, result: &mut WootheeResult) -> bool {
        if agent.contains("CFNetwork/") {
            // This is like iPhone, but only Category and Name are filled
            let d = self.lookup_dataset("iOS");
            if d.is_none() {
                return false;
            }
            let data = d.unwrap();

            result.os = data.name;
            result.category = data.category;
            return true;
        }

        false
    }

    fn challenge_sleipnir<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        if !agent.contains("Sleipnir/") {
            return false;
        }

        let version = match RE_SLEIPNIR_VERSION.captures(agent) {
            Some(caps) => caps.get(1).unwrap().as_str(),
            None => VALUE_UNKNOWN,
        };

        self.populate_dataset(result, "Sleipnir");

        let w = self.lookup_dataset("Win");
        if w.is_none() {
            return false;
        }
        let win = w.unwrap();

        result.version = version;
        result.category = win.category;
        result.os = win.name;

        true
    }

    fn challenge_http_library(&self, agent: &str, result: &mut WootheeResult) -> bool {
        // TODO: wip
        let mut version = "";

        if RX_HTTP_CLIENT.is_match(agent)
            || RX_HTTP_CLIENT_OTHER.is_match(agent)
            || agent.contains("Java(TM) 2 Runtime Environment,")
        {
            version = "Java";
        } else if agent.starts_with("Wget/") {
            version = "wget";
        } else if agent.starts_with("libwww-perl")
            || agent.starts_with("WWW-Mechanize")
            || agent.starts_with("LWP::Simple")
            || agent.starts_with("LWP ")
            || agent.starts_with("lwp-trivial")
        {
            version = "perl";
        } else if agent.starts_with("Ruby") || agent.starts_with("feedzirra") || agent.starts_with("Typhoeus") {
            version = "ruby"
        } else if agent.starts_with("Python-urllib/") || agent.starts_with("Twisted ") {
            version = "python";
        } else if RX_PHP.is_match(agent) || RX_PEAR.is_match(agent) {
            version = "php";
        } else if agent.starts_with("curl/") {
            version = "curl";
        }

        if version.is_empty() {
            return false;
        }

        if !self.populate_dataset(result, "HTTPLibrary") {
            return false;
        }
        result.version = version;

        true
    }

    fn challenge_maybe_rss_reader(&self, agent: &str, result: &mut WootheeResult) -> bool {
        if RX_MAYBE_RSS_PATTERN.is_match(agent)
            || agent.to_lowercase().contains("headline-reader")
            || agent.contains("cococ/")
        {
            return self.populate_dataset(result, "VariousRSSReader");
        }

        false
    }

    fn challenge_maybe_crawler(&self, agent: &str, result: &mut WootheeResult) -> bool {
        if RX_MAYBE_CRAWLER_PATTERN.is_match(agent)
            || RX_MAYBE_CRAWLER_OTHER.is_match(agent)
            || agent.contains("ASP-Ranker Feed Crawler")
            || RX_MAYBE_FEED_PARSER_PATTERN.is_match(agent)
            || RX_MAYBE_WATCHDOG_PATTERN.is_match(agent)
        {
            return self.populate_dataset(result, "VariousCrawler");
        }

        false
    }

    fn challenge_appliance(&self, agent: &str, result: &mut WootheeResult) -> bool {
        if agent.contains("Nintendo DSi;") {
            let d = self.lookup_dataset("NintendoDSi");
            if d.is_none() {
                return false;
            }
            let data = d.unwrap();
            result.category = data.category;
            result.os = data.os;
            return true;
        }

        if agent.contains("Nintendo Wii;") {
            let d = self.lookup_dataset("NintendoWii");
            if d.is_none() {
                return false;
            }
            let data = d.unwrap();
            result.category = data.category;
            result.os = data.os;
            return true;
        }

        false
    }

    fn challenge_misc_os<'a>(&self, agent: &'a str, result: &mut WootheeResult<'a>) -> bool {
        let d = if agent.contains("(Win98;") {
            result.os_version = "98".into();
            self.lookup_dataset("Win98")
        } else if agent.contains("Macintosh; U; PPC;") || agent.contains("Mac_PowerPC") {
            let caps = RX_PPC_OS_VERSION.captures(agent);
            if let Some(c) = caps {
                result.os_version = c.get(1).unwrap().as_str().into();
            }
            self.lookup_dataset("MacOS")
        } else if agent.contains("X11; FreeBSD ") {
            let caps = RX_FREEBSD_OS_VERSION.captures(agent);
            if let Some(c) = caps {
                result.os_version = c.get(1).unwrap().as_str().into();
            }
            self.lookup_dataset("BSD")
        } else if agent.contains("X11; CrOS ") {
            let caps = RX_CHROMEOS_OS_VERSION.captures(agent);
            if let Some(c) = caps {
                result.os_version = c.get(1).unwrap().as_str().into();
            }
            self.lookup_dataset("ChromeOS")
        } else {
            None
        };

        if d.is_none() {
            return false;
        }
        let data = d.unwrap();

        result.category = data.category;
        result.os = data.name;

        true
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    #[test]
    fn test_parse_smoke() {
        let parser = Parser::new();
        match parser.parse("Mozilla/5.0 (Mobile; rv:18.0) Gecko/18.0 Firefox/18.0") {
            Some(result) => assert_eq!(result.category, "smartphone".to_string()),
            None => panic!("invalid"),
        }
    }
}
