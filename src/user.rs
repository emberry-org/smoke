use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct User {
    // DER encoded x509 certificate
    pub cert_data: Vec<u8>,
}
