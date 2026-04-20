//! Firestore REST API v1 via OAuth2 service-account JWT (Datastore scope).

use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;

use crate::booking_rules;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};

const DATASTORE_SCOPE: &str = "https://www.googleapis.com/auth/datastore";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const COLLECTION: &str = "bookings";

#[derive(Debug, Deserialize)]
struct ServiceAccountFile {
    project_id: String,
    client_email: String,
    private_key: String,
}

#[derive(Serialize)]
struct JwtClaims {
    iss: String,
    sub: String,
    scope: String,
    aud: String,
    exp: i64,
    iat: i64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservationDto {
    pub id: String,
    #[serde(rename = "bookedBy")]
    pub booked_by: String,
    pub court: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    pub date: String,
    #[serde(rename = "start_hour")]
    pub start_hour: i64,
    #[serde(rename = "end_hour")]
    pub end_hour: i64,
    #[serde(rename = "messengerUserId")]
    pub messenger_user_id: String,
    pub sport: String,
    pub unit: String,
}

/// Payload for create/update (no document id). Field names match Firestore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservationWrite {
    #[serde(rename = "bookedBy")]
    pub booked_by: String,
    pub court: String,
    pub date: String,
    #[serde(rename = "start_hour")]
    pub start_hour: i64,
    #[serde(rename = "end_hour")]
    pub end_hour: i64,
    #[serde(rename = "messengerUserId")]
    pub messenger_user_id: String,
    pub sport: String,
    pub unit: String,
}

pub struct FirestoreConn {
    http: reqwest::Client,
    sa_path: Option<PathBuf>,
    sa: Option<ServiceAccountFile>,
    access_token: Option<String>,
    token_deadline: Option<Instant>,
}

impl Default for FirestoreConn {
    fn default() -> Self {
        Self {
            http: reqwest::Client::new(),
            sa_path: None,
            sa: None,
            access_token: None,
            token_deadline: None,
        }
    }
}

impl FirestoreConn {
    pub fn is_connected(&self) -> bool {
        self.sa.is_some()
    }

    pub fn project_id(&self) -> Option<&str> {
        self.sa.as_ref().map(|s| s.project_id.as_str())
    }

    pub fn load_service_account_file(&mut self, path: &str) -> Result<(), String> {
        let p = Path::new(path);
        if !p.is_file() {
            return Err(format!("Not a file: {path}"));
        }
        let raw = fs::read_to_string(p).map_err(|e| e.to_string())?;
        let sa: ServiceAccountFile = serde_json::from_str(&raw).map_err(|e| format!("Invalid JSON: {e}"))?;
        if sa.private_key.is_empty() || sa.client_email.is_empty() {
            return Err("Service account JSON missing private_key or client_email".into());
        }
        self.sa = Some(sa);
        self.sa_path = Some(p.to_path_buf());
        self.access_token = None;
        self.token_deadline = None;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.sa = None;
        self.sa_path = None;
        self.access_token = None;
        self.token_deadline = None;
    }

    pub async fn ensure_token(&mut self) -> Result<String, String> {
        let sa = self.sa.as_ref().ok_or("Not connected")?;
        let now = Instant::now();
        if let (Some(tok), Some(dl)) = (&self.access_token, self.token_deadline) {
            if now < dl {
                return Ok(tok.clone());
            }
        }
        let (token, expires_in) = fetch_oauth_token(&self.http, sa).await?;
        let buffer = 300i64;
        let ttl_secs = (expires_in.saturating_sub(buffer)).max(60) as u64;
        self.access_token = Some(token.clone());
        self.token_deadline = Some(now + StdDuration::from_secs(ttl_secs));
        Ok(token)
    }

    fn base_url(&self) -> Result<String, String> {
        let pid = &self.sa.as_ref().ok_or("Not connected")?.project_id;
        Ok(format!(
            "https://firestore.googleapis.com/v1/projects/{pid}/databases/(default)/documents"
        ))
    }

    pub async fn query_week(&mut self, week_start_iso: &str, week_end_iso: &str) -> Result<Vec<ReservationDto>, String> {
        let token = self.ensure_token().await?;
        let base = self.base_url()?;
        let url = format!("{base}:runQuery");
        let body = json!({
            "structuredQuery": {
                "from": [{ "collectionId": COLLECTION }],
                "where": {
                    "compositeFilter": {
                        "op": "AND",
                        "filters": [
                            {
                                "fieldFilter": {
                                    "field": { "fieldPath": "date" },
                                    "op": "GREATER_THAN_OR_EQUAL",
                                    "value": { "stringValue": week_start_iso }
                                }
                            },
                            {
                                "fieldFilter": {
                                    "field": { "fieldPath": "date" },
                                    "op": "LESS_THAN_OR_EQUAL",
                                    "value": { "stringValue": week_end_iso }
                                }
                            }
                        ]
                    }
                }
            }
        });
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| e.to_string())?;
        if !status.is_success() {
            return Err(format!("Firestore query {status}: {text}"));
        }
        let rows: Vec<Value> = serde_json::from_str(&text).map_err(|e| format!("Bad JSON: {e}"))?;
        let mut out = Vec::new();
        for row in rows {
            if let Some(doc) = row.get("document") {
                if let Ok(r) = document_to_reservation(doc) {
                    out.push(r);
                }
            }
        }
        out.sort_by(|a, b| {
            a.date
                .cmp(&b.date)
                .then_with(|| a.start_hour.cmp(&b.start_hour))
        });
        Ok(out)
    }

    pub async fn query_by_date(&mut self, date_iso: &str) -> Result<Vec<ReservationDto>, String> {
        let token = self.ensure_token().await?;
        let base = self.base_url()?;
        let url = format!("{base}:runQuery");
        let body = json!({
            "structuredQuery": {
                "from": [{ "collectionId": COLLECTION }],
                "where": {
                    "fieldFilter": {
                        "field": { "fieldPath": "date" },
                        "op": "EQUAL",
                        "value": { "stringValue": date_iso }
                    }
                }
            }
        });
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| e.to_string())?;
        if !status.is_success() {
            return Err(format!("Firestore query by date {status}: {text}"));
        }
        let rows: Vec<Value> = serde_json::from_str(&text).map_err(|e| format!("Bad JSON: {e}"))?;
        let mut out = Vec::new();
        for row in rows {
            if let Some(doc) = row.get("document") {
                if let Ok(r) = document_to_reservation(doc) {
                    out.push(r);
                }
            }
        }
        out.sort_by(|a, b| a.start_hour.cmp(&b.start_hour));
        Ok(out)
    }

    pub async fn create_reservation(&mut self, mut w: ReservationWrite) -> Result<String, String> {
        booking_rules::validate_sport_and_court(&w.sport, &w.court)?;
        w.messenger_user_id = "admin".to_string();
        let same_day = self.query_by_date(&w.date).await?;
        for ex in &same_day {
            if booking_rules::scheduling_conflict(
                &w.date,
                w.start_hour,
                w.end_hour,
                &w.sport,
                &w.court,
                &ex.id,
                &ex.date,
                ex.start_hour,
                ex.end_hour,
                &ex.sport,
                &ex.court,
                None,
            ) {
                return Err(
                    "Scheduling conflict: that court (or basketball full vs half courts, or full court vs pickleball) is already booked for overlapping hours."
                        .into(),
                );
            }
        }
        let token = self.ensure_token().await?;
        let base = self.base_url()?;
        // REST shape: POST .../documents/{collectionId} (collectionId is a path segment, not ?collectionId=)
        let url = format!("{base}/{COLLECTION}");
        let fields = write_to_fields(&w, true)?;
        let body = json!({ "fields": fields });
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| e.to_string())?;
        if !status.is_success() {
            return Err(format!("Firestore create {status}: {text}"));
        }
        let v: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        let name = v
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or("Missing document name")?;
        parse_doc_id(name)
    }

    pub async fn update_reservation(&mut self, id: &str, w: ReservationWrite) -> Result<(), String> {
        booking_rules::validate_sport_and_court(&w.sport, &w.court)?;
        let same_day = self.query_by_date(&w.date).await?;
        for ex in &same_day {
            if booking_rules::scheduling_conflict(
                &w.date,
                w.start_hour,
                w.end_hour,
                &w.sport,
                &w.court,
                &ex.id,
                &ex.date,
                ex.start_hour,
                ex.end_hour,
                &ex.sport,
                &ex.court,
                Some(id),
            ) {
                return Err(
                    "Scheduling conflict: that court (or basketball full vs half courts, or full court vs pickleball) is already booked for overlapping hours."
                        .into(),
                );
            }
        }
        let token = self.ensure_token().await?;
        let base = self.base_url()?;
        // Do not update messengerUserId (admin tool preserves original messenger on existing rows).
        let url = format!(
            "{base}/{COLLECTION}/{id}?updateMask.fieldPaths=bookedBy&updateMask.fieldPaths=court&updateMask.fieldPaths=date&updateMask.fieldPaths=start_hour&updateMask.fieldPaths=end_hour&updateMask.fieldPaths=sport&updateMask.fieldPaths=unit"
        );
        let fields = write_to_fields_update(&w)?;
        let body = json!({ "fields": fields });
        let resp = self
            .http
            .patch(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Firestore update {status}: {text}"));
        }
        Ok(())
    }

    pub async fn delete_reservation(&mut self, id: &str) -> Result<(), String> {
        let token = self.ensure_token().await?;
        let base = self.base_url()?;
        let url = format!("{base}/{COLLECTION}/{id}");
        let resp = self
            .http
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Firestore delete {status}: {text}"));
        }
        Ok(())
    }
}

async fn fetch_oauth_token(
    http: &reqwest::Client,
    sa: &ServiceAccountFile,
) -> Result<(String, i64), String> {
    let now = Utc::now();
    let exp = now + Duration::seconds(3600);
    let claims = JwtClaims {
        iss: sa.client_email.clone(),
        sub: sa.client_email.clone(),
        scope: DATASTORE_SCOPE.into(),
        aud: TOKEN_URL.into(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };
    let key = EncodingKey::from_rsa_pem(sa.private_key.as_bytes()).map_err(|e| e.to_string())?;
    let jwt = encode(&Header::new(Algorithm::RS256), &claims, &key).map_err(|e| e.to_string())?;
    let params = [
        ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
        ("assertion", jwt.as_str()),
    ];
    let resp = http
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("Token {status}: {text}"));
    }
    let tr: TokenResponse = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    Ok((tr.access_token, tr.expires_in))
}

fn parse_doc_id(name: &str) -> Result<String, String> {
    name.rsplit('/')
        .next()
        .map(|s| s.to_string())
        .ok_or_else(|| "Bad document name".to_string())
}

fn document_to_reservation(doc: &Value) -> Result<ReservationDto, String> {
    let name = doc
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or("missing name")?;
    let id = parse_doc_id(name)?;
    let fields = doc.get("fields").ok_or("missing fields")?;
    Ok(ReservationDto {
        id,
        booked_by: string_field(fields, "bookedBy")?,
        court: string_field(fields, "court")?,
        created_at: timestamp_field_optional(fields, "createdAt"),
        date: string_field(fields, "date")?,
        start_hour: int_field(fields, "start_hour")?,
        end_hour: int_field(fields, "end_hour")?,
        messenger_user_id: string_field(fields, "messengerUserId")?,
        sport: string_field(fields, "sport")?,
        unit: string_field(fields, "unit")?,
    })
}

fn string_field(fields: &Value, key: &str) -> Result<String, String> {
    fields
        .get(key)
        .and_then(|v| v.get("stringValue"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Missing string field {key}"))
}

fn int_field(fields: &Value, key: &str) -> Result<i64, String> {
    let v = fields.get(key).ok_or_else(|| format!("Missing field {key}"))?;
    if let Some(s) = v.get("integerValue").and_then(|x| x.as_str()) {
        return s.parse().map_err(|_| format!("Bad integer {key}"));
    }
    if let Some(i) = v.get("integerValue").and_then(|x| x.as_i64()) {
        return Ok(i);
    }
    Err(format!("Missing integer field {key}"))
}

fn timestamp_field_optional(fields: &Value, key: &str) -> Option<String> {
    fields.get(key).and_then(|v| {
        v.get("timestampValue")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
    })
}

fn write_to_fields(w: &ReservationWrite, is_create: bool) -> Result<Value, String> {
    let mut fields = json!({
        "bookedBy": { "stringValue": w.booked_by },
        "court": { "stringValue": w.court },
        "date": { "stringValue": w.date },
        "start_hour": { "integerValue": w.start_hour.to_string() },
        "end_hour": { "integerValue": w.end_hour.to_string() },
        "messengerUserId": { "stringValue": w.messenger_user_id },
        "sport": { "stringValue": w.sport },
        "unit": { "stringValue": w.unit },
    });
    if is_create {
        let ts = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        fields
            .as_object_mut()
            .unwrap()
            .insert("createdAt".into(), json!({ "timestampValue": ts }));
    }
    Ok(fields)
}

/// PATCH body: excludes `messengerUserId` so existing messenger IDs are never overwritten.
fn write_to_fields_update(w: &ReservationWrite) -> Result<Value, String> {
    Ok(json!({
        "bookedBy": { "stringValue": w.booked_by },
        "court": { "stringValue": w.court },
        "date": { "stringValue": w.date },
        "start_hour": { "integerValue": w.start_hour.to_string() },
        "end_hour": { "integerValue": w.end_hour.to_string() },
        "sport": { "stringValue": w.sport },
        "unit": { "stringValue": w.unit },
    }))
}

pub type SharedConn = Arc<tokio::sync::Mutex<FirestoreConn>>;

pub fn new_shared_conn() -> SharedConn {
    Arc::new(tokio::sync::Mutex::new(FirestoreConn::default()))
}
