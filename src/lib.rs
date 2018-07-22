extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[derive(Debug, Deserialize, PartialEq, Clone)]
/// The categories a Server can be in.
pub enum CategoryType {
    /// A standard VPN server
    Standard,
    /// A VPN server with P2P services allowed.
    P2P,
    /// A VPN server with a obfuscated IP (i.e. floating IP).
    Obfuscated,
    /// A VPN server with a dedicated IP, which is used only by one VPN user at a time.
    Dedicated,
    /// A VPN server with Tor/Onion funcitonality
    Tor,
    /// A VPN server that can be used to connect to another NordVPN server.
    Double,
    /// A VPN server that has a category that is not recognised.
    UnknownServer,
}

impl From<String> for CategoryType {
    fn from(input: String) -> CategoryType {
        match input.as_ref() {
            "Standard VPN servers" => CategoryType::Standard,
            "P2P" => CategoryType::P2P,
            "Double VPN" => CategoryType::Double,
            "Onion Over VPN" => CategoryType::Tor,
            "Obfuscated Servers" => CategoryType::Obfuscated,
            "Dedicated IP servers" => CategoryType::Dedicated,
            server_type => {
                eprintln!("Warning: unknown server type: {}", server_type);
                eprintln!("Please report an issue at https://github.com/editicalu/nordselect");
                CategoryType::UnknownServer
            }
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
struct Category {
    pub name: CategoryType,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
/// All protocols and other features a Server can have.
pub struct Features {
    pub ikev2: bool,
    pub openvpn_udp: bool,
    pub openvpn_tcp: bool,
    pub socks: bool,
    pub proxy: bool,
    pub pptp: bool,
    pub l2tp: bool,
    pub openvpn_xor_udp: bool,
    pub openvpn_xor_tcp: bool,
    pub proxy_cybersec: bool,
    pub proxy_ssl: bool,
    pub proxy_ssl_cybersec: bool,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
/// A server by NordVPN.
pub struct Server {
    /// The country this server is located in.
    flag: String,
    /// The domain of this server.
    pub domain: String,
    /// The current load on this server.
    load: u8,
    /// Categories this server is in.
    categories: Vec<Category>,
    /// Features of the server
    features: Features,
}

/// A list of individual servers.
pub struct Servers {
    /// The actual servers
    servers: Vec<Server>,
}

impl Servers {
    /// Downloads the list of servers from the API.
    pub fn from_api() -> Result<Servers, Box<std::error::Error>> {
        let mut data = reqwest::get("https://api.nordvpn.com/server")?;
        let text = data.text()?;

        Ok(Servers {
            servers: serde_json::from_str(
                // TODO: find a better solution to these expensive replacements.
                &text.replace("Standard VPN servers", "Standard")
                    .replace("Obfuscated Servers", "Obfuscated")
                    .replace("Double VPN", "Double")
                    .replace("Onion Over VPN", "Tor")
                    .replace("Dedicated IP servers", "Dedicated"),
            )?,
        })
    }

    /// Returns the perfect server. This should be called when the filters are applied.
    pub fn get_perfect_server(&self) -> Option<Server> {
        match self.servers.get(0) {
            Some(x) => Some(x.clone()),
            None => None,
        }
    }
}

#[derive(PartialEq)]
/// A protocol to connect to the VPN server.
pub enum Protocol {
    Udp,
    Tcp,
}

/// All filters that can be applied.
impl Servers {
    /// Filters the servers on a certain category.
    pub fn filter_category(&mut self, category: CategoryType) {
        let category_struct = Category { name: category };
        (&mut self.servers).retain(|server| server.categories.contains(&category_struct));
    }

    /// Filters the servers on a certain protocol.
    pub fn filter_protocol(&mut self, protocol: Protocol) {
        match protocol {
            Protocol::Tcp => (&mut self.servers).retain(|server| server.features.openvpn_tcp),
            Protocol::Udp => (&mut self.servers).retain(|server| server.features.openvpn_udp),
        };
    }

    /// Filters the servers on a certain country.
    pub fn filter_country(&mut self, country: &str) {
        (&mut self.servers).retain(|server| server.flag == country)
    }

    /// Sorts the servers on their load.
    pub fn sort_load(&mut self) {
        (&mut self.servers).sort_unstable_by(|x, y| x.load.cmp(&y.load));
    }
}