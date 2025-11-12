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
