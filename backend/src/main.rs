use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod merchants;
use merchants::Merchant;

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    service_key: String,
    cache: Arc<RwLock<Option<Vec<Market>>>>,
    merchants: Arc<RwLock<Option<HashMap<String, Vec<Merchant>>>>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Market {
    name: String,
    market_type: String,
    address_road: Option<String>,
    address_jibun: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    store_count: Option<i32>,
    main_items: Option<String>,
    open_cycle: Option<String>,
    gift_certs: Option<String>,
    homepage: Option<String>,
    has_toilet: Option<String>,
    has_parking: Option<String>,
    open_year: Option<String>,
    phone: Option<String>,
}

// data.go.kr 표준데이터 응답 — 실측 필드명
#[derive(Deserialize, Debug)]
struct ApiItem {
    #[serde(rename = "mrktNm", default)]
    mrkt_nm: String,
    #[serde(rename = "mrktType", default)]
    mrkt_type: Option<String>,
    #[serde(rename = "rdnmadr", default)]
    rdnmadr: Option<String>,
    #[serde(rename = "lnmadr", default)]
    lnmadr: Option<String>,
    #[serde(default)]
    latitude: Option<serde_json::Value>,
    #[serde(default)]
    longitude: Option<serde_json::Value>,
    #[serde(rename = "storNumber", default)]
    stor_number: Option<serde_json::Value>,
    #[serde(rename = "trtmntPrdlst", default)]
    trtmnt_prdlst: Option<String>,
    #[serde(rename = "mrktEstblCycle", default)]
    mrkt_estbl_cycle: Option<String>,
    #[serde(rename = "useGcct", default)]
    use_gcct: Option<String>,
    #[serde(rename = "homepageUrl", default)]
    homepage_url: Option<String>,
    #[serde(rename = "pblicToiletYn", default)]
    pblic_toilet_yn: Option<String>,
    #[serde(rename = "prkplceYn", default)]
    prkplce_yn: Option<String>,
    #[serde(rename = "estblYear", default)]
    estbl_year: Option<serde_json::Value>,
    #[serde(rename = "phoneNumber", default)]
    phone_number: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ApiBody {
    #[serde(default)]
    items: Vec<ApiItem>,
    #[serde(rename = "totalCount", default)]
    total_count: Option<serde_json::Value>,
    #[serde(rename = "numOfRows", default)]
    num_of_rows: Option<serde_json::Value>,
    #[serde(rename = "pageNo", default)]
    page_no: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
struct ApiHeader {
    #[serde(rename = "resultCode")]
    result_code: String,
    #[serde(rename = "resultMsg", default)]
    result_msg: String,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    header: ApiHeader,
    #[serde(default)]
    body: Option<ApiBody>,
}

#[derive(Deserialize, Debug)]
struct ApiEnvelope {
    response: ApiResponse,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,market_map_backend=debug,tower_http=info".into()),
        )
        .init();

    let service_key = std::env::var("MARKET_SERVICE_KEY")
        .context("MARKET_SERVICE_KEY missing in .env")?;
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8770);

    let state = Arc::new(AppState {
        http: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(180))
            .build()?,
        service_key,
        cache: Arc::new(RwLock::new(None)),
        merchants: Arc::new(RwLock::new(None)),
    });

    // 백그라운드로 두 데이터 병렬 로드 — 실패 시 backoff 재시도
    {
        let st = state.clone();
        tokio::spawn(async move {
            for attempt in 1..=5 {
                match load_markets(&st).await {
                    Ok(n) => {
                        tracing::info!("시장 로드 완료: {n}건");
                        return;
                    }
                    Err(e) => {
                        let wait = 5u64.saturating_mul(attempt);
                        tracing::warn!("시장 로드 실패 (시도 {attempt}/5, {wait}s 후 재시도): {e}");
                        tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
                    }
                }
            }
            tracing::error!("시장 로드 5회 모두 실패");
        });
    }
    {
        let st = state.clone();
        tokio::spawn(async move {
            for attempt in 1..=3 {
                match load_merchants(&st).await {
                    Ok(n) => {
                        tracing::info!("가맹점 로드 완료: {n} 시장 그룹");
                        return;
                    }
                    Err(e) => {
                        let wait = 10u64.saturating_mul(attempt);
                        tracing::warn!("가맹점 로드 실패 (시도 {attempt}/3, {wait}s 후 재시도): {e}");
                        tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
                    }
                }
            }
            tracing::error!("가맹점 로드 3회 모두 실패");
        });
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/markets", get(list_markets))
        .route("/api/markets/refresh", get(refresh_markets))
        .route("/api/merchants/by-market/:name", get(merchants_by_market))
        .route("/api/merchants/refresh", get(refresh_merchants))
        .route("/api/merchants/stats", get(merchants_stats))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

#[derive(Deserialize)]
struct ListQuery {
    #[serde(default)]
    r#type: Option<String>,
}

async fn list_markets(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Market>>, ApiError> {
    // 캐시 비어있으면 동기 로드
    {
        let r = state.cache.read().await;
        if r.is_some() {
            let all = r.as_ref().unwrap();
            return Ok(Json(filter(all, q.r#type.as_deref())));
        }
    }
    load_markets(&state)
        .await
        .map_err(|e| ApiError::upstream(format!("load failed: {e}")))?;
    let r = state.cache.read().await;
    let all = r.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
    Ok(Json(filter(all, q.r#type.as_deref())))
}

async fn refresh_markets(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = load_markets(&state)
        .await
        .map_err(|e| ApiError::upstream(format!("refresh failed: {e}")))?;
    Ok(Json(serde_json::json!({ "loaded": n })))
}

#[derive(Deserialize)]
struct MerchantQuery {
    /// 표준데이터 시장의 주소(또는 광역시도) — 동일 시장명 다수 도시 구분용
    #[serde(default)]
    region_hint: Option<String>,
}

async fn merchants_by_market(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(q): Query<MerchantQuery>,
) -> Result<Json<Vec<Merchant>>, ApiError> {
    let hint = q.region_hint.as_deref();
    {
        let r = state.merchants.read().await;
        if let Some(map) = r.as_ref() {
            return Ok(Json(strict_lookup(map, &name, hint)));
        }
    }
    load_merchants(&state)
        .await
        .map_err(|e| ApiError::upstream(format!("load merchants failed: {e}")))?;
    let r = state.merchants.read().await;
    let list = r
        .as_ref()
        .map(|m| strict_lookup(m, &name, hint))
        .unwrap_or_default();
    Ok(Json(list))
}

/// 엄격 매칭: 광역시도 + 시군구 + 시장명 형태 일치
/// 동일 시장명이 여러 도시에 있는 경우(예: "중앙시장")의 cross-contamination 방지.
fn strict_lookup(
    map: &HashMap<String, Vec<Merchant>>,
    needle: &str,
    address_hint: Option<&str>,
) -> Vec<Merchant> {
    let needle = needle.trim();
    if needle.is_empty() {
        return vec![];
    }
    let region = address_hint.and_then(extract_sido);
    let sigungu = address_hint.and_then(extract_sigungu);
    let sigungu_short = sigungu.as_deref().map(strip_sigungu_suffix);

    let mut hits: Vec<Merchant> = Vec::new();
    for (key, list) in map.iter() {
        let merchant_region = list.first().map(|m| m.region.as_str()).unwrap_or("");
        if let Some(r) = region.as_deref() {
            if !regions_match(merchant_region, r) {
                continue;
            }
        }
        if !key_matches(key, needle, sigungu_short.as_deref()) {
            continue;
        }
        hits.extend(list.iter().cloned());
    }
    hits
}

/// `key`(가맹점 시장명)가 검색 시장명과 매칭되는지.
/// - 정확 일치
/// - prefix-only 변형(suffix 추가): "노량진수산시장" → "노량진수산시장골목형상점가"
/// - city prefix 변형: "중앙시장" + 시군구 "안성" → "안성중앙시장" 매칭, 다른 도시 X
fn key_matches(key: &str, needle: &str, sigungu_short: Option<&str>) -> bool {
    if key == needle {
        return true;
    }
    if key.starts_with(needle) {
        return true;
    }
    // 가맹점 시장명이 표준 시장명으로 끝나는 경우 — 시군구 포함 여부 확인
    if let Some(prefix) = key.strip_suffix(needle) {
        if let Some(sg) = sigungu_short {
            return !sg.is_empty() && prefix.contains(sg);
        }
        // 시군구 정보가 없으면 endsWith는 보수적으로 거부
        return false;
    }
    false
}

fn extract_sido(address: &str) -> Option<String> {
    let trimmed = address.trim();
    let first_word = trimmed.split_whitespace().next()?;
    if !is_known_sido(first_word) {
        return None;
    }
    Some(normalize_region(first_word).to_string())
}

fn extract_sigungu(address: &str) -> Option<String> {
    // "경기도 안성시 장기로..." → "안성시"
    // "서울 동작구 노들로..." → "동작구"
    let trimmed = address.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    Some(parts[1].to_string())
}

/// "안성시" → "안성", "동작구" → "동작", "강서군" → "강서"
fn strip_sigungu_suffix(s: &str) -> &str {
    s.trim_end_matches(|c| c == '시' || c == '군' || c == '구')
}

fn is_known_sido(s: &str) -> bool {
    matches!(
        s.trim(),
        "서울" | "서울특별시"
            | "부산" | "부산광역시"
            | "대구" | "대구광역시"
            | "인천" | "인천광역시"
            | "광주" | "광주광역시"
            | "대전" | "대전광역시"
            | "울산" | "울산광역시"
            | "세종" | "세종특별자치시"
            | "경기" | "경기도"
            | "강원" | "강원도" | "강원특별자치도"
            | "충북" | "충청북도"
            | "충남" | "충청남도"
            | "전북" | "전라북도" | "전북특별자치도" | "전북특별차치도"
            | "전남" | "전라남도"
            | "경북" | "경상북도"
            | "경남" | "경상남도"
            | "제주" | "제주특별자치도"
    )
}

fn regions_match(a: &str, b: &str) -> bool {
    normalize_region(a) == normalize_region(b)
}

fn normalize_region(s: &str) -> &str {
    match s.trim() {
        "서울특별시" | "서울" => "서울",
        "부산광역시" | "부산" => "부산",
        "대구광역시" | "대구" => "대구",
        "인천광역시" | "인천" => "인천",
        "광주광역시" | "광주" => "광주",
        "대전광역시" | "대전" => "대전",
        "울산광역시" | "울산" => "울산",
        "세종특별자치시" | "세종" => "세종",
        "경기도" | "경기" => "경기",
        "강원도" | "강원" | "강원특별자치도" => "강원",
        "충청북도" | "충북" => "충북",
        "충청남도" | "충남" => "충남",
        "전라북도" | "전북" | "전북특별자치도" | "전북특별차치도" => "전북",
        "전라남도" | "전남" => "전남",
        "경상북도" | "경북" => "경북",
        "경상남도" | "경남" => "경남",
        "제주특별자치도" | "제주" => "제주",
        x => x,
    }
}


async fn refresh_merchants(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = load_merchants(&state)
        .await
        .map_err(|e| ApiError::upstream(format!("refresh failed: {e}")))?;
    Ok(Json(serde_json::json!({ "groups": n })))
}

async fn merchants_stats(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let r = state.merchants.read().await;
    match r.as_ref() {
        None => Json(serde_json::json!({ "loaded": false })),
        Some(map) => {
            let total: usize = map.values().map(|v| v.len()).sum();
            Json(serde_json::json!({
                "loaded": true,
                "total_merchants": total,
                "market_groups": map.len()
            }))
        }
    }
}

async fn load_merchants(state: &Arc<AppState>) -> anyhow::Result<usize> {
    let map = merchants::load(&state.http).await?;
    let n = map.len();
    let mut w = state.merchants.write().await;
    *w = Some(map);
    Ok(n)
}

fn filter(all: &[Market], type_filter: Option<&str>) -> Vec<Market> {
    match type_filter {
        Some(t) if !t.is_empty() => all.iter().filter(|m| m.market_type.contains(t)).cloned().collect(),
        _ => all.to_vec(),
    }
}

async fn load_markets(state: &Arc<AppState>) -> anyhow::Result<usize> {
    let mut all: Vec<Market> = Vec::new();
    let mut page = 1u32;
    let per_page = 1000u32;
    let mut total: Option<i32> = None;

    loop {
        let body = fetch_page(state, page, per_page).await?;
        let envelope: ApiEnvelope = serde_json::from_str(&body)
            .with_context(|| format!("JSON parse failed. body head: {}", body.chars().take(300).collect::<String>()))?;

        if envelope.response.header.result_code != "00" {
            anyhow::bail!(
                "API returned {} {}",
                envelope.response.header.result_code,
                envelope.response.header.result_msg
            );
        }

        let body = envelope.response.body.unwrap_or(ApiBody {
            items: vec![],
            total_count: None,
            num_of_rows: None,
            page_no: None,
        });

        if total.is_none() {
            total = parse_i32(body.total_count);
        }

        let count = body.items.len();
        for it in body.items {
            all.push(Market {
                name: it.mrkt_nm,
                market_type: it.mrkt_type.unwrap_or_default(),
                address_road: it.rdnmadr.filter(|s| !s.is_empty()),
                address_jibun: it.lnmadr.filter(|s| !s.is_empty()),
                lat: parse_f64(it.latitude),
                lon: parse_f64(it.longitude),
                store_count: parse_i32(it.stor_number),
                main_items: it.trtmnt_prdlst.filter(|s| !s.is_empty()),
                open_cycle: it.mrkt_estbl_cycle.filter(|s| !s.is_empty()),
                gift_certs: it.use_gcct.filter(|s| !s.is_empty()),
                homepage: it.homepage_url.filter(|s| !s.is_empty()),
                has_toilet: it.pblic_toilet_yn.filter(|s| !s.is_empty()),
                has_parking: it.prkplce_yn.filter(|s| !s.is_empty()),
                open_year: parse_i32(it.estbl_year).map(|y| y.to_string()),
                phone: it.phone_number.filter(|s| !s.is_empty()),
            });
        }

        if count < per_page as usize {
            break;
        }
        if let Some(t) = total {
            if (page * per_page) as i32 >= t {
                break;
            }
        }
        page += 1;
        if page > 20 {
            tracing::warn!("페이지 한도 도달, 중단");
            break;
        }
    }

    let mut w = state.cache.write().await;
    let n = all.len();
    *w = Some(all);

    // 시장유형별 카운트 로그
    if let Some(list) = w.as_ref() {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for m in list {
            *counts.entry(m.market_type.clone()).or_insert(0) += 1;
        }
        for (t, c) in counts.iter() {
            tracing::info!("시장유형 '{t}': {c}건");
        }
    }
    Ok(n)
}

async fn fetch_page(state: &AppState, page: u32, num: u32) -> anyhow::Result<String> {
    let key = urlencoding::encode(&state.service_key);
    let url = format!(
        "https://api.data.go.kr/openapi/tn_pubr_public_trdit_mrkt_api\
         ?serviceKey={key}&pageNo={page}&numOfRows={num}&type=json"
    );
    tracing::debug!("fetch page={page}");
    let resp = state.http.get(&url).send().await?;
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        anyhow::bail!("HTTP {status}: {}", body.chars().take(200).collect::<String>());
    }
    Ok(body)
}

fn parse_f64(v: Option<serde_json::Value>) -> Option<f64> {
    match v? {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

fn parse_i32(v: Option<serde_json::Value>) -> Option<i32> {
    match v? {
        serde_json::Value::Number(n) => n.as_i64().map(|x| x as i32),
        serde_json::Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

// --- error handling ---

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn upstream(msg: impl Into<String>) -> Self {
        Self { status: StatusCode::BAD_GATEWAY, message: msg.into() }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        tracing::warn!("api error: {}", self.message);
        (
            self.status,
            Json(serde_json::json!({ "error": self.message })),
        )
            .into_response()
    }
}
