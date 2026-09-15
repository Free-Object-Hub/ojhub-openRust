use axum::http::HeaderMap;
use maxminddb::geoip2;
use std::str::FromStr;
use std::net::IpAddr;

pub fn extract_ip(headers: &HeaderMap) -> String {
    headers
        .get("X-Real-Ip")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}


pub fn get_city(reader: &maxminddb::Reader<Vec<u8>>, ip: &str) -> (String, String) {
    let unknown = ("Unknown".to_string(), "Unknown".to_string());
    let addr = match IpAddr::from_str(ip) {
        Ok(a) => a,
        Err(_) => return unknown,
    };
    let city: geoip2::City = match reader.lookup(addr) {
        Ok(Some(c)) => c,
        _ => return unknown,
    };
    let country = city
        .country
        .as_ref()
        .and_then(|c| c.names.as_ref())
        .and_then(|n| n.get("en"))
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let city_name = city
        .city
        .as_ref()
        .and_then(|c| c.names.as_ref())
        .and_then(|n| n.get("en"))
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    (country, city_name)
}
