use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Clone, Debug)]
pub struct Merchant {
    pub name: String,
    pub market_name: String,
    pub region: String,    // 광역시도
    pub items: String,     // 취급품목
    pub paper: bool,       // 지류형 가맹
    pub digital: bool,     // 디지털형 가맹
    pub registered_year: Option<i32>,
}

// CSV 행 (필드명에 공백·괄호 있어 rename 사용)
#[derive(Deserialize, Debug)]
struct CsvRow {
    #[serde(rename = "가맹점명")]
    name: String,
    #[serde(rename = "소속 시장명(또는 상점가)")]
    market_name: String,
    #[serde(rename = "소재지")]
    region: String,
    #[serde(rename = "취급품목")]
    items: String,
    #[serde(rename = "지류형 가맹 여부")]
    paper: String,
    #[serde(rename = "디지털형 가맹 여부")]
    digital: String,
    #[serde(rename = "등록년도")]
    registered_year: String,
}

const CSV_URL: &str = "https://www.data.go.kr/cmm/cmm/fileDownload.do?atchFileId=FILE_000000003235520&fileDetailSn=1&insertDataPrcus=N";

/// CSV 다운로드 → 시장명별로 그룹핑한 HashMap 반환
pub async fn load(_unused: &reqwest::Client) -> anyhow::Result<HashMap<String, Vec<Merchant>>> {
    tracing::info!("온누리 가맹점 CSV 다운로드 시작");
    // 별도 클라이언트로 — 큰 파일 + 명시적 UA + 더 긴 timeout
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (market-map/0.1)")
        .timeout(std::time::Duration::from_secs(300))
        .build()?;
    let resp = client.get(CSV_URL).send().await?.error_for_status()?;
    let bytes = resp.bytes().await?;
    tracing::info!("CSV 다운로드 완료: {} bytes", bytes.len());

    // UTF-8 BOM 자동 처리
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(bytes.as_ref());

    let mut grouped: HashMap<String, Vec<Merchant>> = HashMap::new();
    let mut total = 0usize;
    let mut bad_rows = 0usize;

    for result in rdr.deserialize::<CsvRow>() {
        match result {
            Ok(r) => {
                let name = r.name.trim().to_string();
                let market_name = r.market_name.trim().to_string();
                if name.is_empty() || market_name.is_empty() {
                    bad_rows += 1;
                    continue;
                }
                let m = Merchant {
                    name,
                    market_name: market_name.clone(),
                    region: r.region.trim().to_string(),
                    items: r.items.trim().to_string(),
                    paper: yn(&r.paper),
                    digital: yn(&r.digital),
                    registered_year: r.registered_year.trim().parse().ok(),
                };
                grouped.entry(market_name).or_insert_with(Vec::new).push(m);
                total += 1;
            }
            Err(e) => {
                bad_rows += 1;
                if bad_rows < 5 {
                    tracing::debug!("CSV row parse skip: {e}");
                }
            }
        }
    }

    tracing::info!(
        "온누리 파싱 완료: 총 {} 가맹점 / {} 시장 그룹 / {} 스킵",
        total,
        grouped.len(),
        bad_rows
    );
    Ok(grouped)
}

fn yn(s: &str) -> bool {
    let t = s.trim().to_uppercase();
    t == "Y" || t == "TRUE" || t == "1"
}
