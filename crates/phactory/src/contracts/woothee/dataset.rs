use super::parser::WootheeResult;
use crate::std::collections::HashMap;

lazy_static! {
    pub static ref DATASET: HashMap<&'static str, WootheeResult<'static>> = {
        let mut dataset = HashMap::new();

        dataset.insert(
            "MSIE",
            WootheeResult {
                name: "Internet Explorer",
                browser_type: "browser",
                category: "",
                os: "",
                os_version: "".into(),
                vendor: "Microsoft",
                version: "",
            },
        );
        dataset.insert(
            "Edge",
            WootheeResult {
                name: "Edge",
                browser_type: "browser",
                category: "",
                os: "",
                os_version: "".into(),
                vendor: "Microsoft",
                version: "",
            },
        );
        dataset.insert(
            "Chrome",
            WootheeResult {
                name: "Chrome",
                browser_type: "browser",
                category: "",
                os: "",
                os_version: "".into(),
                vendor: "Google",
                version: "",
            },
        );
        dataset.insert(
            "Safari",
            WootheeResult {
                name: "Safari",
                browser_type: "browser",
                category: "",
                os: "",
                os_version: "".into(),
                vendor: "Apple",
                version: "",
            },
        );
        dataset.insert(
            "Firefox",
            WootheeResult {
                name: "Firefox",
                browser_type: "browser",
                category: "",
                os: "",
                os_version: "".into(),
                vendor: "Mozilla",
                version: "",
            },
        );
        dataset.insert(
            "Opera",
            WootheeResult {
                name: "Opera",
                browser_type: "browser",
                category: "",
                os: "",
                os_version: "".into(),
                vendor: "Opera",
                version: "",
            },
        );
        dataset.insert(
            "Vivaldi",
            WootheeResult {
                name: "Vivaldi",
                browser_type: "browser",
                category: "",
                os: "",
                os_version: "".into(),
                vendor: "Vivaldi Technologies",
                version: "",
            },
        );
        dataset.insert(
            "Sleipnir",
            WootheeResult {
                name: "Sleipnir",
                browser_type: "browser",
                category: "",
                os: "",
                os_version: "".into(),
                vendor: "Fenrir Inc.",
                version: "",
            },
        );
        dataset.insert(
            "GSA",
            WootheeResult {
                name: "Google Search App",
                browser_type: "browser",
                category: "",
                os: "",
                os_version: "".into(),
                vendor: "Google",
                version: "",
            },
        );
        dataset.insert(
            "Webview",
            WootheeResult {
                name: "Webview",
                browser_type: "browser",
                category: "",
                os: "",
                os_version: "".into(),
                vendor: "OS vendor",
                version: "",
            },
        );
        dataset.insert(
            "YaBrowser",
            WootheeResult {
                name: "Yandex Browser",
                browser_type: "browser",
                category: "",
                os: "",
                os_version: "".into(),
                vendor: "Yandex",
                version: "",
            },
        );
        dataset.insert(
            "Win",
            WootheeResult {
                name: "Windows UNKNOWN Ver",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "Win10",
            WootheeResult {
                name: "Windows 10",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "Win8.1",
            WootheeResult {
                name: "Windows 8.1",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "Win8",
            WootheeResult {
                name: "Windows 8",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "Win7",
            WootheeResult {
                name: "Windows 7",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "WinVista",
            WootheeResult {
                name: "Windows Vista",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "WinXP",
            WootheeResult {
                name: "Windows XP",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "Win2000",
            WootheeResult {
                name: "Windows 2000",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "WinNT4",
            WootheeResult {
                name: "Windows NT 4.0",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "WinMe",
            WootheeResult {
                name: "Windows Me",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "Win98",
            WootheeResult {
                name: "Windows 98",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "Win95",
            WootheeResult {
                name: "Windows 95",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "WinPhone",
            WootheeResult {
                name: "Windows Phone OS",
                browser_type: "os",
                category: "smartphone",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "WinCE",
            WootheeResult {
                name: "Windows CE",
                browser_type: "os",
                category: "smartphone",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "OSX",
            WootheeResult {
                name: "Mac OSX",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "MacOS",
            WootheeResult {
                name: "Mac OS Classic",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "Linux",
            WootheeResult {
                name: "Linux",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "BSD",
            WootheeResult {
                name: "BSD",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "ChromeOS",
            WootheeResult {
                name: "ChromeOS",
                browser_type: "os",
                category: "pc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "Android",
            WootheeResult {
                name: "Android",
                browser_type: "os",
                category: "smartphone",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "iPhone",
            WootheeResult {
                name: "iPhone",
                browser_type: "os",
                category: "smartphone",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "iPad",
            WootheeResult {
                name: "iPad",
                browser_type: "os",
                category: "smartphone",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "iPod",
            WootheeResult {
                name: "iPod",
                browser_type: "os",
                category: "smartphone",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "iOS",
            WootheeResult {
                name: "iOS",
                browser_type: "os",
                category: "smartphone",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "FirefoxOS",
            WootheeResult {
                name: "Firefox OS",
                browser_type: "os",
                category: "smartphone",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "BlackBerry",
            WootheeResult {
                name: "BlackBerry",
                browser_type: "os",
                category: "smartphone",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "BlackBerry10",
            WootheeResult {
                name: "BlackBerry 10",
                browser_type: "os",
                category: "smartphone",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "docomo",
            WootheeResult {
                name: "docomo",
                browser_type: "full",
                category: "mobilephone",
                os: "docomo",
                os_version: "".into(),
                vendor: "docomo",
                version: "",
            },
        );
        dataset.insert(
            "au",
            WootheeResult {
                name: "au by KDDI",
                browser_type: "full",
                category: "mobilephone",
                os: "au",
                os_version: "".into(),
                vendor: "au",
                version: "",
            },
        );
        dataset.insert(
            "SoftBank",
            WootheeResult {
                name: "SoftBank Mobile",
                browser_type: "full",
                category: "mobilephone",
                os: "SoftBank",
                os_version: "".into(),
                vendor: "SoftBank",
                version: "",
            },
        );
        dataset.insert(
            "willcom",
            WootheeResult {
                name: "WILLCOM",
                browser_type: "full",
                category: "mobilephone",
                os: "WILLCOM",
                os_version: "".into(),
                vendor: "WILLCOM",
                version: "",
            },
        );
        dataset.insert(
            "jig",
            WootheeResult {
                name: "jig browser",
                browser_type: "full",
                category: "mobilephone",
                os: "jig",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "emobile",
            WootheeResult {
                name: "emobile",
                browser_type: "full",
                category: "mobilephone",
                os: "emobile",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "SymbianOS",
            WootheeResult {
                name: "SymbianOS",
                browser_type: "full",
                category: "mobilephone",
                os: "SymbianOS",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "MobileTranscoder",
            WootheeResult {
                name: "Mobile Transcoder",
                browser_type: "full",
                category: "mobilephone",
                os: "Mobile Transcoder",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "Nintendo3DS",
            WootheeResult {
                name: "Nintendo 3DS",
                browser_type: "full",
                category: "appliance",
                os: "Nintendo 3DS",
                os_version: "".into(),
                vendor: "Nintendo",
                version: "",
            },
        );
        dataset.insert(
            "NintendoDSi",
            WootheeResult {
                name: "Nintendo DSi",
                browser_type: "full",
                category: "appliance",
                os: "Nintendo DSi",
                os_version: "".into(),
                vendor: "Nintendo",
                version: "",
            },
        );
        dataset.insert(
            "NintendoWii",
            WootheeResult {
                name: "Nintendo Wii",
                browser_type: "full",
                category: "appliance",
                os: "Nintendo Wii",
                os_version: "".into(),
                vendor: "Nintendo",
                version: "",
            },
        );
        dataset.insert(
            "NintendoWiiU",
            WootheeResult {
                name: "Nintendo Wii U",
                browser_type: "full",
                category: "appliance",
                os: "Nintendo Wii U",
                os_version: "".into(),
                vendor: "Nintendo",
                version: "",
            },
        );
        dataset.insert(
            "PSP",
            WootheeResult {
                name: "PlayStation Portable",
                browser_type: "full",
                category: "appliance",
                os: "PlayStation Portable",
                os_version: "".into(),
                vendor: "Sony",
                version: "",
            },
        );
        dataset.insert(
            "PSVita",
            WootheeResult {
                name: "PlayStation Vita",
                browser_type: "full",
                category: "appliance",
                os: "PlayStation Vita",
                os_version: "".into(),
                vendor: "Sony",
                version: "",
            },
        );
        dataset.insert(
            "PS3",
            WootheeResult {
                name: "PlayStation 3",
                browser_type: "full",
                category: "appliance",
                os: "PlayStation 3",
                os_version: "".into(),
                vendor: "Sony",
                version: "",
            },
        );
        dataset.insert(
            "PS4",
            WootheeResult {
                name: "PlayStation 4",
                browser_type: "full",
                category: "appliance",
                os: "PlayStation 4",
                os_version: "".into(),
                vendor: "Sony",
                version: "",
            },
        );
        dataset.insert(
            "Xbox360",
            WootheeResult {
                name: "Xbox 360",
                browser_type: "full",
                category: "appliance",
                os: "Xbox 360",
                os_version: "".into(),
                vendor: "Microsoft",
                version: "",
            },
        );
        dataset.insert(
            "XboxOne",
            WootheeResult {
                name: "Xbox One",
                browser_type: "full",
                category: "appliance",
                os: "Xbox One",
                os_version: "".into(),
                vendor: "Microsoft",
                version: "",
            },
        );
        dataset.insert(
            "DigitalTV",
            WootheeResult {
                name: "InternetTVBrowser",
                browser_type: "full",
                category: "appliance",
                os: "DigitalTV",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "SafariRSSReader",
            WootheeResult {
                name: "Safari RSSReader",
                browser_type: "full",
                category: "misc",
                os: "",
                os_version: "".into(),
                vendor: "Apple",
                version: "",
            },
        );
        dataset.insert(
            "GoogleDesktop",
            WootheeResult {
                name: "Google Desktop",
                browser_type: "full",
                category: "misc",
                os: "",
                os_version: "".into(),
                vendor: "Google",
                version: "",
            },
        );
        dataset.insert(
            "WindowsRSSReader",
            WootheeResult {
                name: "Windows RSSReader",
                browser_type: "full",
                category: "misc",
                os: "",
                os_version: "".into(),
                vendor: "Microsoft",
                version: "",
            },
        );
        dataset.insert(
            "VariousRSSReader",
            WootheeResult {
                name: "RSSReader",
                browser_type: "full",
                category: "misc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "HTTPLibrary",
            WootheeResult {
                name: "HTTP Library",
                browser_type: "full",
                category: "misc",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "GoogleBot",
            WootheeResult {
                name: "Googlebot",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "GoogleBotMobile",
            WootheeResult {
                name: "Googlebot Mobile",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "GoogleMediaPartners",
            WootheeResult {
                name: "Google Mediapartners",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "GoogleFeedFetcher",
            WootheeResult {
                name: "Google Feedfetcher",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "GoogleAppEngine",
            WootheeResult {
                name: "Google AppEngine",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "GoogleWebPreview",
            WootheeResult {
                name: "Google Web Preview",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "YahooSlurp",
            WootheeResult {
                name: "Yahoo! Slurp",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "YahooJP",
            WootheeResult {
                name: "Yahoo! Japan",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "YahooPipes",
            WootheeResult {
                name: "Yahoo! Pipes",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "Baiduspider",
            WootheeResult {
                name: "Baiduspider",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "msnbot",
            WootheeResult {
                name: "msnbot",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "bingbot",
            WootheeResult {
                name: "bingbot",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "BingPreview",
            WootheeResult {
                name: "BingPreview",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "Yeti",
            WootheeResult {
                name: "Naver Yeti",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "FeedBurner",
            WootheeResult {
                name: "Google FeedBurner",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "facebook",
            WootheeResult {
                name: "facebook",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "twitter",
            WootheeResult {
                name: "twitter",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "trendictionbot",
            WootheeResult {
                name: "trendiction",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "mixi",
            WootheeResult {
                name: "mixi",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "IndyLibrary",
            WootheeResult {
                name: "Indy Library",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "ApplePubSub",
            WootheeResult {
                name: "Apple iCloud",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "Genieo",
            WootheeResult {
                name: "Genieo Web Filter",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "topsyButterfly",
            WootheeResult {
                name: "topsy Butterfly",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "rogerbot",
            WootheeResult {
                name: "SeoMoz rogerbot",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "AhrefsBot",
            WootheeResult {
                name: "ahref AhrefsBot",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "radian6",
            WootheeResult {
                name: "salesforce radian6",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "Hatena",
            WootheeResult {
                name: "Hatena",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "goo",
            WootheeResult {
                name: "goo",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "livedoorFeedFetcher",
            WootheeResult {
                name: "livedoor FeedFetcher",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset.insert(
            "VariousCrawler",
            WootheeResult {
                name: "misc crawler",
                browser_type: "full",
                category: "crawler",
                os: "",
                os_version: "".into(),
                vendor: "",
                version: "",
            },
        );
        dataset
    };
}

use lazy_static;
