use anyhow::{anyhow, Result};
use cloudflare::endpoints::dns::dns;
use cloudflare::framework;
use cloudflare::framework::client::blocking_api::HttpApiClient;
use log::{debug, info, trace};
use reqwest::blocking::get;
use std::marker::PhantomData;
use std::net::Ipv4Addr;

/// Reference to API of which local IP
const PUBLIC_IP_API: &str = "https://api.ipify.org";

/// A state when IP addresses are uninitialised or unchecked
pub struct Unvalidated;
/// A state when IP addresses do not match
pub struct Invalid;
/// A state when IP addresses match
pub struct Valid;

/// Server configurator enables configuring of Cloudflare to match local IP addresses
pub struct ServerConfigurator<State = Unvalidated> {
    state: PhantomData<State>,
    actual_ip: Ipv4Addr,
    configured_ip: Ipv4Addr,
    config_client: HttpApiClient,
    config_site: String,
    config_zone: String,
    config_record_id: String,
}

/// A result type used to capture the state following validations.
/// Where the types can be valid or invalid
pub enum ValidationResult {
    Valid(ServerConfigurator<Valid>),
    Invalid(ServerConfigurator<Invalid>),
}

trait Validateable<T> {
    fn update_ips(&mut self) -> Result<()>;
    fn validate(self) -> T;
}

trait Configurable<T> {
    fn reconfigure_ip(&mut self) -> Result<()>;
    fn reconfigure(self) -> T;
}

impl ServerConfigurator {
    /// Initialise a new `ServerConfigurator`
    pub fn new(token: &str, site: &str, zone: &str) -> Self {
        debug!(
            "Created server configurator: configuring {} in zone {}",
            site, zone
        );
        let credentials = framework::auth::Credentials::UserAuthToken {
            token: token.to_string(),
        };
        let config = framework::client::ClientConfig::default();
        let environment = framework::Environment::Production;
        let config_client = HttpApiClient::new(credentials, config, environment)
            .expect("failed to initialise token");
        trace!(
            "Cloudflare client successfully configured with token: {}",
            token
        );
        Self {
            state: Default::default(),
            actual_ip: Ipv4Addr::new(0, 0, 0, 0),
            configured_ip: Ipv4Addr::new(0, 0, 0, 0),
            config_client,
            config_site: site.to_string(),
            config_zone: zone.to_string(),
            config_record_id: "".to_string(),
        }
    }
}

impl<T> ServerConfigurator<T> {
    fn log_ips(&self) {
        debug!(
            "actual ip: {:?} configured ip: {:?}",
            self.actual_ip, self.configured_ip
        );
    }
}

fn get_configured_ip<T>(server_config: &ServerConfigurator<T>) -> Result<(Ipv4Addr, String)> {
    let endpoint = cloudflare::endpoints::dns::dns::ListDnsRecords {
        zone_identifier: &server_config.config_zone,
        params: cloudflare::endpoints::dns::dns::ListDnsRecordsParams {
            direction: Some(framework::OrderDirection::Ascending),
            ..Default::default()
        },
    };
    let response = server_config.config_client.request(&endpoint)?;
    trace!("Response returned from Cloudflare {:?}", response);

    // Response contains a list of results in the results field
    // Within this each result contains a name that can be used to identify the record
    // Filter the records by the name
    let dns_record = response
        .result
        .iter()
        .filter(|&r| r.name == server_config.config_site);
    // Then each record contains arbitary volumes of DNS Content
    // For A records, this is only the IP address.
    // Filter out all other records returning only the IP
    // Then collect first (and hopefully only) record from the iterator
    let ip_address = dns_record
        .clone()
        .filter_map(|r| match r.content {
            dns::DnsContent::A { content: ip } => Some(ip),
            _ => None,
        })
        .next()
        .ok_or_else(|| anyhow!("no configured IP address value found"))?;

    let record_id = dns_record
        .map(|r| r.id.clone())
        .next()
        .ok_or_else(|| anyhow!("no configured record ID value found"))?;

    debug!(
        "Record found for {}: IP is {} and record ID is {}",
        server_config.config_site, ip_address, record_id
    );

    Ok((ip_address, record_id))
}

fn get_actual_ip() -> Result<Ipv4Addr> {
    Ok(get(PUBLIC_IP_API)?.text()?.parse()?)
}

impl Validateable<ValidationResult> for ServerConfigurator<Unvalidated> {
    fn update_ips(&mut self) -> Result<()> {
        self.actual_ip = get_actual_ip()?;
        (self.configured_ip, self.config_record_id) = get_configured_ip(self)?;
        Ok(())
    }

    fn validate(mut self) -> ValidationResult {
        self.log_ips();
        info!("Validating IPs");
        let _ = self.update_ips();
        match self.actual_ip == self.configured_ip {
            true => {
                info!("IP configuration is valid");
                ValidationResult::Valid(ServerConfigurator {
                    state: PhantomData,
                    actual_ip: self.actual_ip,
                    configured_ip: self.configured_ip,
                    config_client: self.config_client,
                    config_site: self.config_site,
                    config_zone: self.config_zone,
                    config_record_id: self.config_record_id,
                })
            }
            false => {
                info!("IP configuration is invalid");
                ValidationResult::Invalid(ServerConfigurator {
                    state: PhantomData,
                    actual_ip: self.actual_ip,
                    configured_ip: self.configured_ip,
                    config_client: self.config_client,
                    config_site: self.config_site,
                    config_zone: self.config_zone,
                    config_record_id: self.config_record_id,
                })
            }
        }
    }
}

fn configure_ip<T>(server_config: &ServerConfigurator<T>) -> Result<Ipv4Addr> {
    let endpoint = cloudflare::endpoints::dns::dns::UpdateDnsRecord {
        zone_identifier: &server_config.config_zone,
        identifier: &server_config.config_record_id,
        params: cloudflare::endpoints::dns::dns::UpdateDnsRecordParams {
            name: &server_config.config_site,
            content: cloudflare::endpoints::dns::dns::DnsContent::A {
                content: server_config.actual_ip,
            },
            ttl: Some(1),
            proxied: Some(true),
        },
    };
    let response = server_config.config_client.request(&endpoint)?;
    trace!("Response returned from Cloudflare {:?}", response);

    let configured_ip = match response.result.content {
        dns::DnsContent::A { content: ip } => Some(ip),
        _ => None,
    }
    .ok_or_else(|| anyhow!("no valid IP found to be configured"))?;
    Ok(configured_ip)
}

impl Configurable<ServerConfigurator<Unvalidated>> for ServerConfigurator<Invalid> {
    fn reconfigure_ip(&mut self) -> Result<()> {
        self.configured_ip = configure_ip(self)?;
        Ok(())
    }

    fn reconfigure(mut self) -> ServerConfigurator<Unvalidated> {
        self.log_ips();
        info!("Reconfiguring IP Addresses");
        let _ = self.reconfigure_ip();
        ServerConfigurator {
            state: PhantomData,
            actual_ip: self.actual_ip,
            configured_ip: self.configured_ip,
            config_client: self.config_client,
            config_site: self.config_site,
            config_zone: self.config_zone,
            config_record_id: self.config_record_id,
        }
    }
}

impl ServerConfigurator<Valid> {
    fn complete(self) {
        self.log_ips();
        info!("IP configured correctly");
    }
}

/// A function to run through the state machine of the `ServerConfigurator`
pub fn configure(server_config: ServerConfigurator<Unvalidated>) -> Result<()> {
    let mut server_config = server_config.validate();
    loop {
        match server_config {
            ValidationResult::Invalid(invalid) => server_config = invalid.reconfigure().validate(),
            ValidationResult::Valid(valid) => {
                valid.complete();
                break;
            }
        }
    }
    Ok(())
}
