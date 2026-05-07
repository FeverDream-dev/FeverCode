use std::fmt;
use std::path::Path;
use std::str::FromStr;

use anyhow::{Result, anyhow, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LicenseTier {
    #[default]
    Community,
    Pro,
    Team,
    Enterprise,
}

impl fmt::Display for LicenseTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LicenseTier::Community => write!(f, "community"),
            LicenseTier::Pro => write!(f, "pro"),
            LicenseTier::Team => write!(f, "team"),
            LicenseTier::Enterprise => write!(f, "enterprise"),
        }
    }
}

impl FromStr for LicenseTier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "community" => Ok(LicenseTier::Community),
            "pro" => Ok(LicenseTier::Pro),
            "team" => Ok(LicenseTier::Team),
            "enterprise" => Ok(LicenseTier::Enterprise),
            other => Err(anyhow!("unknown license tier: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseKey {
    pub tier: LicenseTier,
    pub email: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub machine_id: Option<String>,
    pub seats: Option<u32>,
    pub signature: String,
}

impl LicenseKey {
    pub fn generate(tier: LicenseTier, email: &str, secret: &[u8]) -> Self {
        let issued_at = Utc::now();
        let payload = format!("{}{}{}", tier, email, issued_at.timestamp());
        let signature = compute_hmac(secret, payload.as_bytes());
        LicenseKey {
            tier,
            email: email.to_string(),
            issued_at,
            expires_at: None,
            machine_id: None,
            seats: None,
            signature,
        }
    }

    pub fn verify(&self, secret: &[u8]) -> bool {
        let payload = format!("{}{}{}", self.tier, self.email, self.issued_at.timestamp());
        let expected = compute_hmac(secret, payload.as_bytes());
        hmac_equal(&self.signature, &expected)
    }

    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => Utc::now() > exp,
            None => false,
        }
    }

    pub fn is_valid(&self, secret: &[u8]) -> bool {
        self.verify(secret) && !self.is_expired()
    }

    pub fn encode(&self) -> Result<String> {
        let json = serde_json::to_string(self)?;
        Ok(BASE64.encode(json.as_bytes()))
    }

    pub fn decode(encoded: &str) -> Result<Self> {
        let bytes = BASE64.decode(encoded.trim())?;
        let json = String::from_utf8(bytes)?;
        let key: LicenseKey = serde_json::from_str(&json)?;
        Ok(key)
    }

    pub fn display_tier(&self) -> String {
        match self.tier {
            LicenseTier::Community => "Community".to_string(),
            LicenseTier::Pro => "Pro".to_string(),
            LicenseTier::Team => match self.seats {
                Some(n) => format!("Team ({} seats)", n),
                None => "Team".to_string(),
            },
            LicenseTier::Enterprise => "Enterprise".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    CloudSync,
    Analytics,
    CustomSouls,
    EarlyAccess,
    PersistentMemory,
    SharedSouls,
    CentralBilling,
    AuditLog,
    AdminDashboard,
    TeamAnalytics,
    Sso,
    OnPrem,
    CustomModels,
    ComplianceReporting,
    DedicatedSupport,
}

impl Feature {
    pub fn min_tier(&self) -> LicenseTier {
        match self {
            Feature::CloudSync
            | Feature::Analytics
            | Feature::CustomSouls
            | Feature::EarlyAccess
            | Feature::PersistentMemory => LicenseTier::Pro,

            Feature::SharedSouls
            | Feature::CentralBilling
            | Feature::AuditLog
            | Feature::AdminDashboard
            | Feature::TeamAnalytics => LicenseTier::Team,

            Feature::Sso
            | Feature::OnPrem
            | Feature::CustomModels
            | Feature::ComplianceReporting
            | Feature::DedicatedSupport => LicenseTier::Enterprise,
        }
    }
}

fn tier_rank(tier: LicenseTier) -> u8 {
    match tier {
        LicenseTier::Community => 0,
        LicenseTier::Pro => 1,
        LicenseTier::Team => 2,
        LicenseTier::Enterprise => 3,
    }
}

fn tier_allows(current: LicenseTier, required: LicenseTier) -> bool {
    tier_rank(current) >= tier_rank(required)
}

pub struct LicenseManager {
    license: Option<LicenseKey>,
    secret: Vec<u8>,
}

impl LicenseManager {
    pub fn new(secret: &[u8]) -> Self {
        LicenseManager {
            license: None,
            secret: secret.to_vec(),
        }
    }

    pub fn load_from_file(path: &Path) -> Result<Option<LicenseKey>> {
        if !path.exists() {
            return Ok(None);
        }
        let contents = std::fs::read_to_string(path)?;
        let trimmed = contents.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        let key = LicenseKey::decode(trimmed)?;
        Ok(Some(key))
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        match &self.license {
            Some(key) => {
                let encoded = key.encode()?;
                std::fs::write(path, encoded)?;
                Ok(())
            }
            None => {
                bail!("no active license to save");
            }
        }
    }

    pub fn activate(&mut self, encoded_key: &str) -> Result<LicenseTier> {
        let key = LicenseKey::decode(encoded_key)?;
        if !key.verify(&self.secret) {
            bail!("license signature verification failed");
        }
        if key.is_expired() {
            bail!("license has expired");
        }
        let tier = key.tier;
        self.license = Some(key);
        Ok(tier)
    }

    pub fn deactivate(&mut self) -> Result<()> {
        if self.license.is_none() {
            bail!("no active license to deactivate");
        }
        self.license = None;
        Ok(())
    }

    pub fn current_tier(&self) -> LicenseTier {
        match &self.license {
            Some(key) => key.tier,
            None => LicenseTier::Community,
        }
    }

    pub fn check_feature(&self, feature: Feature) -> bool {
        tier_allows(self.current_tier(), feature.min_tier())
    }
}

fn compute_hmac(secret: &[u8], payload: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(payload);
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    hex_encode(code_bytes.as_slice())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hmac_equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut result: u8 = 0;
    for i in 0..a_bytes.len() {
        result |= a_bytes[i] ^ b_bytes[i];
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    const TEST_SECRET: &[u8] = b"test-secret-key-12345";

    #[test]
    fn test_generate_and_verify_key() {
        let key = LicenseKey::generate(LicenseTier::Pro, "user@example.com", TEST_SECRET);
        assert!(key.verify(TEST_SECRET));
        assert!(!key.verify(b"wrong-secret"));
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let mut key = LicenseKey::generate(LicenseTier::Enterprise, "admin@corp.com", TEST_SECRET);
        key.expires_at = Some(Utc::now() + Duration::days(365));
        key.machine_id = Some("hw-abc123".to_string());
        key.seats = Some(50);
        let encoded = key.encode().unwrap();
        let decoded = LicenseKey::decode(&encoded).unwrap();
        assert_eq!(decoded.tier, key.tier);
        assert_eq!(decoded.email, key.email);
        assert_eq!(decoded.issued_at, key.issued_at);
        assert_eq!(decoded.expires_at, key.expires_at);
        assert_eq!(decoded.machine_id, key.machine_id);
        assert_eq!(decoded.seats, key.seats);
        assert_eq!(decoded.signature, key.signature);
    }

    #[test]
    fn test_expired_key_is_invalid() {
        let mut key = LicenseKey::generate(LicenseTier::Pro, "user@example.com", TEST_SECRET);
        key.expires_at = Some(Utc::now() - Duration::hours(1));
        assert!(key.is_expired());
        assert!(!key.is_valid(TEST_SECRET));
    }

    #[test]
    fn test_community_tier_is_default() {
        let tier = LicenseTier::default();
        assert_eq!(tier, LicenseTier::Community);
    }

    #[test]
    fn test_feature_gate_pro_requires_pro_tier() {
        assert_eq!(Feature::CloudSync.min_tier(), LicenseTier::Pro);
        assert_eq!(Feature::Analytics.min_tier(), LicenseTier::Pro);
        assert_eq!(Feature::CustomSouls.min_tier(), LicenseTier::Pro);
        assert_eq!(Feature::EarlyAccess.min_tier(), LicenseTier::Pro);
        assert_eq!(Feature::PersistentMemory.min_tier(), LicenseTier::Pro);
    }

    #[test]
    fn test_feature_gate_team_requires_team_tier() {
        assert_eq!(Feature::SharedSouls.min_tier(), LicenseTier::Team);
        assert_eq!(Feature::CentralBilling.min_tier(), LicenseTier::Team);
        assert_eq!(Feature::AuditLog.min_tier(), LicenseTier::Team);
        assert_eq!(Feature::AdminDashboard.min_tier(), LicenseTier::Team);
        assert_eq!(Feature::TeamAnalytics.min_tier(), LicenseTier::Team);
    }

    #[test]
    fn test_activate_with_valid_key() {
        let key = LicenseKey::generate(LicenseTier::Pro, "user@example.com", TEST_SECRET);
        let encoded = key.encode().unwrap();
        let mut mgr = LicenseManager::new(TEST_SECRET);
        let tier = mgr.activate(&encoded).unwrap();
        assert_eq!(tier, LicenseTier::Pro);
        assert_eq!(mgr.current_tier(), LicenseTier::Pro);
    }

    #[test]
    fn test_activate_with_invalid_signature_fails() {
        let key = LicenseKey::generate(LicenseTier::Pro, "user@example.com", TEST_SECRET);
        let mut bad_key = key.clone();
        bad_key.signature = "0000000000000000".to_string();
        let encoded = bad_key.encode().unwrap();
        let mut mgr = LicenseManager::new(TEST_SECRET);
        assert!(mgr.activate(&encoded).is_err());
    }

    #[test]
    fn test_activate_with_expired_key_fails() {
        let mut key = LicenseKey::generate(LicenseTier::Pro, "user@example.com", TEST_SECRET);
        key.expires_at = Some(Utc::now() - Duration::hours(1));
        let encoded = key.encode().unwrap();
        let mut mgr = LicenseManager::new(TEST_SECRET);
        assert!(mgr.activate(&encoded).is_err());
    }

    #[test]
    fn test_deactivate_removes_license() {
        let key = LicenseKey::generate(LicenseTier::Pro, "user@example.com", TEST_SECRET);
        let encoded = key.encode().unwrap();
        let mut mgr = LicenseManager::new(TEST_SECRET);
        mgr.activate(&encoded).unwrap();
        assert_eq!(mgr.current_tier(), LicenseTier::Pro);
        mgr.deactivate().unwrap();
        assert_eq!(mgr.current_tier(), LicenseTier::Community);
    }

    #[test]
    fn test_display_tier_formats() {
        let mut key = LicenseKey::generate(LicenseTier::Team, "u@e.com", TEST_SECRET);
        key.seats = Some(10);
        assert_eq!(key.display_tier(), "Team (10 seats)");

        key.tier = LicenseTier::Enterprise;
        assert_eq!(key.display_tier(), "Enterprise");

        key.tier = LicenseTier::Pro;
        assert_eq!(key.display_tier(), "Pro");

        key.tier = LicenseTier::Community;
        assert_eq!(key.display_tier(), "Community");
    }

    #[test]
    fn test_tier_from_str() {
        assert_eq!("pro".parse::<LicenseTier>().unwrap(), LicenseTier::Pro);
        assert_eq!("PRO".parse::<LicenseTier>().unwrap(), LicenseTier::Pro);
        assert_eq!("Team".parse::<LicenseTier>().unwrap(), LicenseTier::Team);
        assert_eq!(
            "ENTERPRISE".parse::<LicenseTier>().unwrap(),
            LicenseTier::Enterprise
        );
        assert_eq!(
            "community".parse::<LicenseTier>().unwrap(),
            LicenseTier::Community
        );
        assert!("unknown".parse::<LicenseTier>().is_err());
    }

    #[test]
    fn test_license_save_load_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("license.key");

        let key = LicenseKey::generate(LicenseTier::Enterprise, "admin@corp.com", TEST_SECRET);
        let encoded = key.encode().unwrap();

        let mut mgr = LicenseManager::new(TEST_SECRET);
        mgr.activate(&encoded).unwrap();
        mgr.save_to_file(&path).unwrap();

        let loaded = LicenseManager::load_from_file(&path).unwrap().unwrap();
        assert!(loaded.verify(TEST_SECRET));
        assert_eq!(loaded.tier, LicenseTier::Enterprise);
        assert_eq!(loaded.email, "admin@corp.com");
    }

    #[test]
    fn test_community_cannot_use_pro_features() {
        let mgr = LicenseManager::new(TEST_SECRET);
        assert!(!mgr.check_feature(Feature::CloudSync));
        assert!(!mgr.check_feature(Feature::Analytics));
        assert!(!mgr.check_feature(Feature::CustomSouls));
        assert!(!mgr.check_feature(Feature::SharedSouls));
        assert!(!mgr.check_feature(Feature::Sso));
    }

    #[test]
    fn test_enterprise_tier_allows_all_features() {
        let key = LicenseKey::generate(LicenseTier::Enterprise, "e@c.com", TEST_SECRET);
        let encoded = key.encode().unwrap();
        let mut mgr = LicenseManager::new(TEST_SECRET);
        mgr.activate(&encoded).unwrap();
        assert!(mgr.check_feature(Feature::CloudSync));
        assert!(mgr.check_feature(Feature::SharedSouls));
        assert!(mgr.check_feature(Feature::Sso));
    }

    #[test]
    fn test_pro_tier_allows_pro_but_not_team_features() {
        let key = LicenseKey::generate(LicenseTier::Pro, "p@c.com", TEST_SECRET);
        let encoded = key.encode().unwrap();
        let mut mgr = LicenseManager::new(TEST_SECRET);
        mgr.activate(&encoded).unwrap();
        assert!(mgr.check_feature(Feature::CloudSync));
        assert!(mgr.check_feature(Feature::Analytics));
        assert!(!mgr.check_feature(Feature::SharedSouls));
        assert!(!mgr.check_feature(Feature::Sso));
    }
}
