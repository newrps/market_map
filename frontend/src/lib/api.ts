export interface Market {
  name: string;
  market_type: string;
  address_road: string | null;
  address_jibun: string | null;
  lat: number | null;
  lon: number | null;
  store_count: number | null;
  main_items: string | null;
  open_cycle: string | null;
  gift_certs: string | null;
  homepage: string | null;
  has_toilet: string | null;
  has_parking: string | null;
  open_year: string | null;
  phone: string | null;
}

const BASE = '/api';

export async function fetchMarkets(typeFilter?: string): Promise<Market[]> {
  const url = typeFilter ? `${BASE}/markets?type=${encodeURIComponent(typeFilter)}` : `${BASE}/markets`;
  const res = await fetch(url);
  if (!res.ok) throw new Error(`markets: ${res.status}`);
  return res.json();
}

export interface Merchant {
  name: string;
  market_name: string;
  region: string;
  items: string;
  paper: boolean;
  digital: boolean;
  registered_year: number | null;
}

export async function fetchMerchantsByMarket(marketName: string, regionHint?: string): Promise<Merchant[]> {
  const url = new URL(`${BASE}/merchants/by-market/${encodeURIComponent(marketName)}`, location.origin);
  if (regionHint) url.searchParams.set('region_hint', regionHint);
  const res = await fetch(url.pathname + url.search);
  if (!res.ok) throw new Error(`merchants: ${res.status}`);
  return res.json();
}

// 시장유형 → 큰 카테고리: 상설장 / 정기장(3·4·5일장)
export type TypeCategory = '상설장' | '정기장';

export function typeCategory(t: string): TypeCategory {
  // "상설장+4일장" 같은 복합형은 상설로 분류
  return t.includes('상설') ? '상설장' : '정기장';
}

export function typeColor(t: string): string {
  return typeCategory(t) === '상설장' ? '#2E7D32' : '#FB8C00';
}
