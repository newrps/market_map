# 🏪 전통시장 지도 (market-map)

전국 전통시장 위치·정보를 Leaflet 지도에 표시. data.go.kr 표준데이터 사용.

## 구성

```
backend/   Rust (axum) — data.go.kr 프록시 + 메모리 캐시
frontend/  SvelteKit + Leaflet
```

## 데이터 출처

- 전국전통시장표준데이터 (소상공인시장진흥공단)
- API: `https://api.data.go.kr/openapi/tn_pubr_public_trdit_mrkt_api`
- 갱신: 연 1회

## 실행 (개발)

```bash
# backend
cd backend
cp .env.example .env  # MARKET_SERVICE_KEY 채우기
cargo run             # → http://127.0.0.1:8770

# frontend
cd frontend
npm install
npm run dev           # → http://localhost:5174
```

## 배포 (NAS)

```bash
docker compose up -d --build
# → http://localhost:18081
```

## API 엔드포인트

- `GET /api/health`
- `GET /api/markets` — 전체 시장 목록 (메모리 캐시)
- `GET /api/markets?type=수산시장` — 시장 유형 필터

## 환경변수

| 키 | 설명 |
|----|------|
| `MARKET_SERVICE_KEY` | data.go.kr 인증키 (Encoding 버전) |
| `PORT` | 백엔드 포트 (기본 8770) |
| `HOST_PORT` | 프론트 호스트 포트 (기본 18081) |
| `RUST_LOG` | 로그 레벨 (info) |
