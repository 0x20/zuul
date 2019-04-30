use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SocketPaths {
    // Default: inproc://modem_urc
    modem_urc: Option<String>,
    // Default: inproc://modem_rpc
    modem_rpc: Option<String>,
    // Default: inproc://event
    event: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MqttConfig {
    pub enable: bool,
    pub server: String,
    pub port: u16,
    pub client_id: String,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    pub sockets: SocketPaths,
    pub mqtt: Option<MqttConfig>,
}

impl SocketPaths {
    pub fn modem_urc(&self) -> &str {
        self.modem_urc
            .as_ref()
            .map(String::as_str)
            .unwrap_or("inproc://modem_urc")
    }

    pub fn modem_rpc(&self) -> &str {
        self.modem_rpc
            .as_ref()
            .map(String::as_str)
            .unwrap_or("inproc://modem_rpc")
    }

    pub fn event(&self) -> &str {
        self.event
            .as_ref()
            .map(String::as_str)
            .unwrap_or("inproc://event")
    }
}
