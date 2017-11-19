// Osmium is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Osmium is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Osmium.  If not, see <http://www.gnu.org/licenses/>.

use http2::settings as http2_settings;

pub struct ServerSettings {
    host: String,
    port: u16,
    security: Option<SecuritySettings>,
    // TODO read these settings on connection start.
    http2_settings: Option<Vec<http2_settings::SettingsParameter>>
}

pub struct SecuritySettings {
    ssl_cert_path: String,
    ssl_cert_pass: String
}

impl SecuritySettings {
    pub fn default() -> Self {
        SecuritySettings {
            ssl_cert_path: String::from("tests/cert.pfx"),
            ssl_cert_pass: String::from("asdf")
        }
    }

    pub fn get_ssl_cert_path(&self) -> &str {
        self.ssl_cert_path.as_ref()
    }

    pub fn set_ssl_cert_path(&mut self, ssl_cert_path: String) {
        self.ssl_cert_path = ssl_cert_path;
    }

    pub fn get_ssl_cert_pass(&self) -> &str {
        self.ssl_cert_pass.as_ref()
    }
}

impl ServerSettings {
    /// Create a default settings
    ///
    /// By default the settings are to connect to localhost:8080 with no security.
    pub fn default() -> Self {
        ServerSettings {
            host: String::from("0.0.0.0"),
            port: 8080,
            security: None,
            http2_settings: None
        }
    }

    pub fn get_host(&self) -> &str {
        self.host.as_ref()
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    // TODO this whole feature is a bit messy now.
    pub fn is_use_security(&self) -> bool {
        self.security.is_some()
    }

    pub fn get_security(&self) -> &SecuritySettings {
        if let Some(ref security) = self.security {
            return security;
        }
        
        panic!("Server not correctly configured. Expected security settings but none were found");
    }

    pub fn set_security(&mut self, security: SecuritySettings) {
        self.security = Some(security);
    }
}
