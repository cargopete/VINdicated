use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::SourceData;

pub mod tier_a;
pub mod tier_b;
pub mod tier_s;
pub mod universal;

/// Every data source implements this trait.
#[async_trait]
pub trait VehicleSource: Send + Sync {
    /// Short machine-readable ID, e.g. "nhtsa_vpic", "nl_rdw"
    fn id(&self) -> &'static str;
    /// ISO 3166-1 alpha-2, or "XX" for universal / "EU" for EU-wide sources
    fn country(&self) -> &'static str;
    /// Human-readable display name
    fn name(&self) -> &'static str;
    /// How long to cache a successful response
    fn cache_ttl(&self) -> Duration;
    /// Fetch data by VIN
    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError>;
}

/// Registry of all active sources, built at startup.
pub struct SourceRegistry {
    pub sources: Vec<Arc<dyn VehicleSource>>,
}

impl SourceRegistry {
    pub fn build() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("VINdicate/0.1 (+https://vindicate-app.com)")
            .build()
            .expect("Failed to build HTTP client");

        let sources: Vec<Arc<dyn VehicleSource>> = vec![
            // Universal (always active, no auth required)
            Arc::new(universal::nhtsa_vpic::NhtsaVpic::new(client.clone())),
            Arc::new(universal::nhtsa_recalls::NhtsaRecalls::new(client.clone())),
            Arc::new(universal::eu_safety_gate::EuSafetyGate::new(client.clone())),
            // Tier S — official free APIs, no auth
            Arc::new(tier_s::nl_rdw::NlRdw::new(client.clone())),
            Arc::new(tier_s::dk_dmr::DkDmr::new(client.clone())),
            Arc::new(tier_s::se_transportstyrelsen::SeTransportstyrelsen::new(client.clone())),
            Arc::new(tier_s::no_vegvesen::NoVegvesen::new(client.clone())),
            Arc::new(tier_s::pl_historiapojazdu::PlHistoriaPojazdu::new(client.clone())),
            Arc::new(tier_s::cz_kontrolatachometru::CzKontrolaTachometru::new(client.clone())),
            Arc::new(tier_s::sk_stkonline::SkStkOnline::new(client.clone())),
            Arc::new(tier_s::si_avtolog::SiAvtolog::new(client.clone())),
            Arc::new(tier_s::ee_transpordiamet::EeTranspordiamet::new(client.clone())),
            Arc::new(tier_s::lv_csdd::LvCsdd::new(client.clone())),
            Arc::new(tier_s::ua_hsc::UaHsc::new(client.clone())),
            // Tier A
            Arc::new(tier_a::hr_hak::HrHak::new(client.clone())),
            Arc::new(tier_a::lt_regitra::LtRegitra::new(client.clone())),
            Arc::new(tier_a::is_samgongustofa::IsSamgongustofa::new(client.clone())),
            Arc::new(tier_a::es_dgt::EsDgt::new(client.clone())),
            Arc::new(tier_a::it_aci::ItAci::new(client.clone())),
            // Tier B (limited / form-based)
            Arc::new(tier_b::de_kba::DeKba::new(client.clone())),
            Arc::new(tier_b::fr_histovec::FrHistovec::new(client.clone())),
            Arc::new(tier_b::bg_mobile::BgMobile::new(client.clone())),
        ];

        Self { sources }
    }
}
