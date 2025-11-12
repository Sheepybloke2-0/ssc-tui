use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// NASA SSC API client
pub struct SscClient {
    base_url: String,
    client: reqwest::Client,
    api_key: Option<String>,
    /// Cached satellites for quick lookup
    satellites_cache: Option<HashMap<String, Observatory>>,
}

/// XML Response from /observatories endpoint
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ObservatoryResponse {
    #[serde(rename = "Observatory", default)]
    observatory: Vec<Observatory>,
}

/// Observatory (Satellite) information matching NASA SSC XML schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Observatory {
    pub id: String,
    pub name: String,
    #[serde(rename = "Resolution")]
    pub resolution: u32, // in seconds
    #[serde(rename = "StartTime")]
    pub start_time: String,
    #[serde(rename = "EndTime")]
    pub end_time: String,
    #[serde(rename = "ResourceId", default)]
    pub resource_id: Option<String>,
    #[serde(rename = "GroupId", default)]
    pub group_id: Vec<String>,
}

impl Observatory {
    /// Get resolution as human-readable string
    pub fn resolution_str(&self) -> String {
        match self.resolution {
            60 => "1min".to_string(),
            180 => "3min".to_string(),
            300 => "5min".to_string(),
            720 => "12min".to_string(),
            3600 => "1hr".to_string(),
            _ => format!("{}s", self.resolution),
        }
    }

    /// Parse start time to DateTime
    pub fn start_time_parsed(&self) -> Result<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.start_time)
            .map(|dt| dt.with_timezone(&Utc))
            .context("Failed to parse start time")
    }

    /// Parse end time to DateTime
    pub fn end_time_parsed(&self) -> Result<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.end_time)
            .map(|dt| dt.with_timezone(&Utc))
            .context("Failed to parse end time")
    }

    /// Get a short description of the observatory
    pub fn description(&self) -> String {
        let start_year = self.start_time_parsed()
            .map(|dt| dt.format("%Y").to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        let status = if let Ok(end_time) = self.end_time_parsed() {
            if end_time > Utc::now() {
                "Active"
            } else {
                "Historical"
            }
        } else {
            "Unknown"
        };

        format!("{} | {} | Since {}", status, self.resolution_str(), start_year)
    }
}

/// Position data for a satellite
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SatellitePosition {
    pub time: DateTime<Utc>,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

/// Query parameters for satellite location
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LocationQuery {
    pub satellite_ids: Vec<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub coordinate_system: CoordinateSystem,
}

/// Coordinate system options
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum CoordinateSystem {
    Geo,
    Gm,
    Gse,
    Gsm,
    Sm,
}

impl SscClient {
    /// Create a new SSC API client, loading API_KEY from environment
    pub fn new() -> Result<Self> {
        // Load .env file if it exists
        dotenv::dotenv().ok();

        let api_key = env::var("API_KEY").ok();

        Ok(Self {
            base_url: "https://sscweb.gsfc.nasa.gov/WS/sscr/2".to_string(),
            client: reqwest::Client::new(),
            api_key,
            satellites_cache: None,
        })
    }

    /// Get list of available satellites from NASA SSC API
    pub async fn get_satellites(&mut self) -> Result<Vec<Observatory>> {
        // Return cached data if available
        if let Some(cache) = &self.satellites_cache {
            return Ok(cache.values().cloned().collect());
        }

        let url = format!("{}/observatories", self.base_url);

        let mut request = self.client
            .get(&url)
            .header("Accept", "application/xml");

        // Add API key if available
        if let Some(api_key) = &self.api_key {
            request = request.header("X-API-Key", api_key);
        }

        let response = request
            .send()
            .await
            .context("Failed to fetch observatories from NASA SSC API")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "API request failed with status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        let xml_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        let observatory_response: ObservatoryResponse = quick_xml::de::from_str(&xml_text)
            .context("Failed to parse XML response")?;

        // Build HashMap for quick lookups
        let mut cache = HashMap::new();
        for obs in &observatory_response.observatory {
            cache.insert(obs.id.clone(), obs.clone());
        }

        self.satellites_cache = Some(cache);

        Ok(observatory_response.observatory)
    }

    /// Get satellite information by ID (uses cache if available)
    pub async fn get_satellite_info(&mut self, satellite_id: &str) -> Result<Option<Observatory>> {
        // Ensure cache is populated
        if self.satellites_cache.is_none() {
            self.get_satellites().await?;
        }

        Ok(self.satellites_cache
            .as_ref()
            .and_then(|cache| cache.get(satellite_id).cloned()))
    }

    /// Search satellites by name (partial match, case-insensitive)
    pub async fn search_satellites(&mut self, query: &str) -> Result<Vec<Observatory>> {
        // Ensure cache is populated
        if self.satellites_cache.is_none() {
            self.get_satellites().await?;
        }

        let query_lower = query.to_lowercase();

        Ok(self.satellites_cache
            .as_ref()
            .map(|cache| {
                cache.values()
                    .filter(|obs| {
                        obs.name.to_lowercase().contains(&query_lower) ||
                        obs.id.to_lowercase().contains(&query_lower)
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Get count of cached satellites
    pub fn cached_count(&self) -> usize {
        self.satellites_cache
            .as_ref()
            .map(|cache| cache.len())
            .unwrap_or(0)
    }

    /// Query satellite locations for a time range
    /// TODO: Implement actual API call
    pub async fn query_locations(&self, _query: &LocationQuery) -> Result<Vec<SatellitePosition>> {
        // Stub: Return empty results
        Ok(vec![])
    }

    /// Get satellite trajectory for visualization
    /// TODO: Implement actual API call
    pub async fn get_trajectory(
        &self,
        _satellite_id: &str,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<SatellitePosition>> {
        // Stub: Return empty trajectory
        Ok(vec![])
    }

    /// Calculate conjunction between satellites
    /// TODO: Implement actual analysis
    pub async fn find_conjunctions(
        &self,
        _satellite_ids: Vec<String>,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
        _threshold_km: f64,
    ) -> Result<Vec<ConjunctionEvent>> {
        // Stub: Return empty results
        Ok(vec![])
    }

    /// Get ground track for a satellite
    /// TODO: Implement actual calculation
    pub async fn get_ground_track(
        &self,
        _satellite_id: &str,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<(f64, f64)>> {
        // Stub: Return empty ground track
        Ok(vec![])
    }
}

/// Conjunction event between satellites
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ConjunctionEvent {
    pub time: DateTime<Utc>,
    pub satellite1: String,
    pub satellite2: String,
    pub distance_km: f64,
}

impl Default for SscClient {
    fn default() -> Self {
        Self::new().expect("Failed to create SSC client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observatory_resolution_str() {
        let obs = create_test_observatory(60);
        assert_eq!(obs.resolution_str(), "1min");

        let obs = create_test_observatory(180);
        assert_eq!(obs.resolution_str(), "3min");

        let obs = create_test_observatory(300);
        assert_eq!(obs.resolution_str(), "5min");

        let obs = create_test_observatory(720);
        assert_eq!(obs.resolution_str(), "12min");

        let obs = create_test_observatory(3600);
        assert_eq!(obs.resolution_str(), "1hr");

        let obs = create_test_observatory(90);
        assert_eq!(obs.resolution_str(), "90s");
    }

    #[test]
    fn test_observatory_start_time_parsed_valid() {
        let obs = Observatory {
            id: "test".to_string(),
            name: "Test".to_string(),
            resolution: 60,
            start_time: "2020-01-15T10:30:00Z".to_string(),
            end_time: "2030-01-01T00:00:00Z".to_string(),
            resource_id: None,
            group_id: vec![],
        };

        let parsed = obs.start_time_parsed().unwrap();
        assert_eq!(parsed.format("%Y-%m-%d").to_string(), "2020-01-15");
    }

    #[test]
    fn test_observatory_start_time_parsed_invalid() {
        let obs = Observatory {
            id: "test".to_string(),
            name: "Test".to_string(),
            resolution: 60,
            start_time: "invalid-date".to_string(),
            end_time: "2030-01-01T00:00:00Z".to_string(),
            resource_id: None,
            group_id: vec![],
        };

        assert!(obs.start_time_parsed().is_err());
    }

    #[test]
    fn test_observatory_end_time_parsed_valid() {
        let obs = Observatory {
            id: "test".to_string(),
            name: "Test".to_string(),
            resolution: 60,
            start_time: "2020-01-01T00:00:00Z".to_string(),
            end_time: "2025-06-30T23:59:59Z".to_string(),
            resource_id: None,
            group_id: vec![],
        };

        let parsed = obs.end_time_parsed().unwrap();
        assert_eq!(parsed.format("%Y-%m-%d").to_string(), "2025-06-30");
    }

    #[test]
    fn test_observatory_end_time_parsed_invalid() {
        let obs = Observatory {
            id: "test".to_string(),
            name: "Test".to_string(),
            resolution: 60,
            start_time: "2020-01-01T00:00:00Z".to_string(),
            end_time: "not-a-date".to_string(),
            resource_id: None,
            group_id: vec![],
        };

        assert!(obs.end_time_parsed().is_err());
    }

    #[test]
    fn test_observatory_description_active() {
        let obs = Observatory {
            id: "test".to_string(),
            name: "Test Satellite".to_string(),
            resolution: 60,
            start_time: "2020-01-01T00:00:00Z".to_string(),
            end_time: "2099-12-31T23:59:59Z".to_string(), // Far in the future
            resource_id: None,
            group_id: vec![],
        };

        let desc = obs.description();
        assert!(desc.contains("Active"));
        assert!(desc.contains("1min"));
        assert!(desc.contains("2020"));
    }

    #[test]
    fn test_observatory_description_historical() {
        let obs = Observatory {
            id: "test".to_string(),
            name: "Test Satellite".to_string(),
            resolution: 300,
            start_time: "2010-01-01T00:00:00Z".to_string(),
            end_time: "2015-01-01T00:00:00Z".to_string(), // In the past
            resource_id: None,
            group_id: vec![],
        };

        let desc = obs.description();
        assert!(desc.contains("Historical"));
        assert!(desc.contains("5min"));
        assert!(desc.contains("2010"));
    }

    #[test]
    fn test_observatory_description_invalid_date() {
        let obs = Observatory {
            id: "test".to_string(),
            name: "Test Satellite".to_string(),
            resolution: 60,
            start_time: "invalid-date".to_string(),
            end_time: "also-invalid".to_string(),
            resource_id: None,
            group_id: vec![],
        };

        let desc = obs.description();
        assert!(desc.contains("Unknown"));
    }

    #[test]
    fn test_ssc_client_new() {
        let client = SscClient::new();
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.base_url, "https://sscweb.gsfc.nasa.gov/WS/sscr/2");
        assert_eq!(client.cached_count(), 0);
        assert!(client.satellites_cache.is_none());
    }

    #[test]
    fn test_ssc_client_default() {
        // This should not panic
        let _client = SscClient::default();
    }

    #[test]
    fn test_ssc_client_cached_count_empty() {
        let client = SscClient::new().unwrap();
        assert_eq!(client.cached_count(), 0);
    }

    #[test]
    fn test_ssc_client_cached_count_with_data() {
        let mut client = SscClient::new().unwrap();

        // Manually populate cache for testing
        let mut cache = HashMap::new();
        cache.insert("sat1".to_string(), create_test_observatory(60));
        cache.insert("sat2".to_string(), create_test_observatory(300));
        cache.insert("sat3".to_string(), create_test_observatory(3600));

        client.satellites_cache = Some(cache);

        assert_eq!(client.cached_count(), 3);
    }

    #[test]
    fn test_observatory_serialization() {
        let obs = Observatory {
            id: "test-sat".to_string(),
            name: "Test Satellite".to_string(),
            resolution: 60,
            start_time: "2020-01-01T00:00:00Z".to_string(),
            end_time: "2030-01-01T00:00:00Z".to_string(),
            resource_id: Some("spase://NASA/Observatory/TestSat".to_string()),
            group_id: vec!["group1".to_string(), "group2".to_string()],
        };

        // Test that it can be serialized
        let json = serde_json::to_string(&obs);
        assert!(json.is_ok());
    }

    #[test]
    fn test_observatory_deserialization() {
        let json = r#"{
            "Id": "test-sat",
            "Name": "Test Satellite",
            "Resolution": 60,
            "StartTime": "2020-01-01T00:00:00Z",
            "EndTime": "2030-01-01T00:00:00Z",
            "ResourceId": "spase://NASA/Observatory/TestSat",
            "GroupId": ["group1", "group2"]
        }"#;

        let obs: Result<Observatory, _> = serde_json::from_str(json);
        assert!(obs.is_ok());

        let obs = obs.unwrap();
        assert_eq!(obs.id, "test-sat");
        assert_eq!(obs.name, "Test Satellite");
        assert_eq!(obs.resolution, 60);
        assert_eq!(obs.resource_id, Some("spase://NASA/Observatory/TestSat".to_string()));
        assert_eq!(obs.group_id, vec!["group1", "group2"]);
    }

    #[test]
    fn test_observatory_deserialization_optional_fields() {
        let json = r#"{
            "Id": "test-sat",
            "Name": "Test Satellite",
            "Resolution": 60,
            "StartTime": "2020-01-01T00:00:00Z",
            "EndTime": "2030-01-01T00:00:00Z"
        }"#;

        let obs: Result<Observatory, _> = serde_json::from_str(json);
        assert!(obs.is_ok());

        let obs = obs.unwrap();
        assert_eq!(obs.resource_id, None);
        assert_eq!(obs.group_id, Vec::<String>::new());
    }

    #[test]
    fn test_coordinate_system_variants() {
        // Just verify all variants exist and can be created
        let _geo = CoordinateSystem::Geo;
        let _gm = CoordinateSystem::Gm;
        let _gse = CoordinateSystem::Gse;
        let _gsm = CoordinateSystem::Gsm;
        let _sm = CoordinateSystem::Sm;
    }

    // Helper function
    fn create_test_observatory(resolution: u32) -> Observatory {
        Observatory {
            id: "test".to_string(),
            name: "Test Observatory".to_string(),
            resolution,
            start_time: "2020-01-01T00:00:00Z".to_string(),
            end_time: "2030-01-01T00:00:00Z".to_string(),
            resource_id: None,
            group_id: vec![],
        }
    }
}
