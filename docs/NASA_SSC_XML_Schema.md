# NASA SSC Web Services XML Schema Documentation

This document describes the XML schema used by NASA's Satellite Situation Center (SSC) Web Services API.

## Base URL

```
https://sscweb.gsfc.nasa.gov/WS/sscr/2
```

## Get Observatories Endpoint

### Request

**URL:** `/observatories`

**Method:** GET

**Headers:**
- `Accept: application/xml` (required for XML response)
- `X-API-Key: <your_key>` (optional, if API key is required)

**Full Example:**
```
GET https://sscweb.gsfc.nasa.gov/WS/sscr/2/observatories
Accept: application/xml
```

### Response Structure

The response is an XML document following the `ObservatoryResponse` schema:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<ObservatoryResponse>
    <Observatory>
        <Id>ace</Id>
        <Name>ACE</Name>
        <Resolution>720</Resolution>
        <StartTime>1997-08-25T17:48:00.000+00:00</StartTime>
        <EndTime>2025-12-29T23:49:00.000+00:00</EndTime>
        <ResourceId>spase://SMWG/Observatory/ACE</ResourceId>
        <GroupId><!-- Optional parent mission/group ID --></GroupId>
    </Observatory>
    <Observatory>
        <!-- More observatories... -->
    </Observatory>
</ObservatoryResponse>
```

## ObservatoryResponse Fields

### Root Element: `ObservatoryResponse`

Contains an array of `Observatory` elements.

### Observatory Element

Each `Observatory` element represents a satellite/spacecraft and contains:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `Id` | string | Yes | Unique identifier (lowercase, alphanumeric) |
| `Name` | string | Yes | Display name of the satellite/observatory |
| `Resolution` | integer | Yes | Data resolution in seconds. Common values: 60, 180, 300, 720, 3600 |
| `StartTime` | ISO 8601 DateTime | Yes | Mission start time (RFC 3339 format) |
| `EndTime` | ISO 8601 DateTime | Yes | Mission end time or predicted trajectory end |
| `ResourceId` | string | No | SPASE resource identifier (e.g., `spase://SMWG/Observatory/ACE`) |
| `GroupId` | array[string] | No | Parent mission/group identifiers for multi-satellite missions |

### Resolution Values

| Seconds | Human Readable |
|---------|----------------|
| 60 | 1 minute |
| 180 | 3 minutes |
| 300 | 5 minutes |
| 720 | 12 minutes |
| 3600 | 1 hour |

### DateTime Format

All datetime fields follow RFC 3339 / ISO 8601 format with timezone:

```
YYYY-MM-DDTHH:MM:SS.sss+00:00
```

Example: `1997-08-25T17:48:00.000+00:00`

## Example Response (Excerpt)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<ObservatoryResponse>
    <Observatory>
        <Id>iss</Id>
        <Name>International Space Station</Name>
        <Resolution>60</Resolution>
        <StartTime>2000-02-01T00:00:00.000+00:00</StartTime>
        <EndTime>2025-12-31T23:59:00.000+00:00</EndTime>
        <ResourceId>spase://SMWG/Observatory/ISS</ResourceId>
    </Observatory>

    <Observatory>
        <Id>ace</Id>
        <Name>ACE</Name>
        <Resolution>720</Resolution>
        <StartTime>1997-08-25T17:48:00.000+00:00</StartTime>
        <EndTime>2025-12-29T23:49:00.000+00:00</EndTime>
        <ResourceId>spase://SMWG/Observatory/ACE</ResourceId>
    </Observatory>

    <Observatory>
        <Id>geotail</Id>
        <Name>Geotail</Name>
        <Resolution>60</Resolution>
        <StartTime>1992-07-24T00:00:00.000+00:00</StartTime>
        <EndTime>2022-11-28T00:00:00.000+00:00</EndTime>
        <ResourceId>spase://SMWG/Observatory/Geotail</ResourceId>
    </Observatory>
</ObservatoryResponse>
```

## Dataset Coverage

The API provides data for **300+ satellites and observatories** including:

- **Historical missions:** Dating back to Sputnik 1 (1957)
- **Current missions:** Active spacecraft with real-time tracking
- **Future missions:** Predicted trajectories extending to 2040

## Status Codes

| Code | Meaning |
|------|---------|
| 200 | Success - Observatory data returned |
| 304 | Not Modified (when using If-None-Match header) |
| 406 | Not Acceptable - Invalid Accept header |
| 500 | Server Error |

## Caching

The API supports HTTP caching via ETag headers:

**Request Header:**
```
If-None-Match: "<etag-value>"
```

**Response Headers:**
```
ETag: "<etag-value>"
Cache-Control: max-age=3600
```

## Alternative Endpoint: SPASE Observatories

**URL:** `/spaseObservatories`

Same format as `/observatories` but includes SPASE ResourceID identifiers for all observatories.

## Rust Implementation

See `src/ssc.rs` for the Rust implementation using:
- `quick-xml` for XML parsing
- `serde` for deserialization
- `reqwest` for HTTP requests

The `Observatory` struct matches this XML schema with appropriate serde annotations.
