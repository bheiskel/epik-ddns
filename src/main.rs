use config::{Config, ConfigError};
use cron_job::{CronJob, Job};
use dns_lookup::lookup_host;
use log::{debug, error, info};
use reqwest::{blocking, header};
use serde::Deserialize;
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::time::Duration;

static EPIK_API_BASE_URL: &str = "https://usersapiv2.epik.com/v2/";
static IPIFY_REQUEST_URL: &str = "https://api.ipify.org?format=json";
static DEFAULT_SCHEDULE: &str = "0 */15 * * * * ";

#[derive(Deserialize, Debug, Clone)]
#[allow(unused)]
struct Settings {
    signature: String,
    domain: String,
    hostnames: Vec<String>,
    dryrun: Option<bool>,
    updateschedule: Option<String>,
}
impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(
                config::Environment::with_prefix("DDNS")
                    .separator("_")
                    .list_separator(",")
                    .with_list_parse_key("hostnames")
                    .try_parsing(true),
            )
            .build()?;
        s.try_deserialize()
    }
}

#[derive(Deserialize, Debug)]
struct Ip {
    ip: String,
}

fn get_external_ip() -> Result<Ipv4Addr, String> {
    debug!("Getting external ip");
    match blocking::get(IPIFY_REQUEST_URL).unwrap().json::<Ip>() {
        Err(err) => Err(format!("{}", err)),
        Ok(ip) => match ip.ip.parse() {
            Err(err) => Err(format!("{}", err)),
            Ok(ext_ip) => Ok(ext_ip),
        },
    }
}

fn update_dns_record(host: &String, external_ip: &String, ddns_req_url: &String) {
    let mut map = HashMap::new();
    map.insert("hostname", host);
    map.insert("value", external_ip);

    let client = blocking::Client::new();

    let resp = client
        .post(ddns_req_url)
        .header(header::USER_AGENT, "")
        .json(&map)
        .send()
        .unwrap();
    debug!("response: {:?}", resp);
    if resp.status().is_success() {
        info!("DNS record for {host} changed to {external_ip}");
    } else {
        error!(
            "Failed to change DNS record, status code: {}",
            resp.status().as_u16()
        );
    }
}

struct UpdateJob {
    settings: Settings,
}
impl Job for UpdateJob {
    fn run(&mut self) {
        let ddns_req_url: String = format!(
            "{}ddns/set-ddns?SIGNATURE={}",
            EPIK_API_BASE_URL, self.settings.signature,
        )
        .to_string();
        debug!("ddns_req_url: {ddns_req_url}");

        let cur_ip: Vec<std::net::IpAddr> = lookup_host(&self.settings.domain).unwrap();
        info!("DNS lookup for {}: {}", self.settings.domain, cur_ip[0]);

        match get_external_ip() {
            Ok(external_ip) => {
                info!("External IP: {external_ip}");
                if self.settings.dryrun.is_none() {
                    if external_ip != cur_ip[0] {
                        for host in self.settings.hostnames.iter() {
                            update_dns_record(&host, &external_ip.to_string(), &ddns_req_url);
                        }
                    } else {
                        info!(
                            "{} is already pointing to: {}",
                            self.settings.domain, external_ip
                        );
                    }
                }
            }
            Err(error) => error!("{}", error),
        }
    }
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let mut settings = Settings::new().unwrap();
    debug!("{settings:?}");
    if settings.dryrun.is_some() {
        info!("Running dry run, hostrecords will not be changed");
    }

    if settings.updateschedule.is_none() {
        settings.updateschedule = Some(DEFAULT_SCHEDULE.to_string());
    }
    let update_job = UpdateJob {
        settings: settings.clone(),
    };

    let mut cron = CronJob::default();

    cron.new_job(settings.updateschedule.unwrap().as_str(), update_job);

    cron.start();
    loop {
        std::thread::sleep(Duration::from_millis(500));
    }
}
