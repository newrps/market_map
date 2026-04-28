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

// 시장유형별 색상
export function typeColor(t: string): string {
  if (t.includes('수산')) return '#0277BD';
  if (t.includes('상설')) return '#2E7D32';
  if (t.includes('정기') || t.includes('5일') || t.includes('정기시장')) return '#FB8C00';
  if (t.includes('농수산')) return '#1B5E20';
  return '#558B2F';
}
