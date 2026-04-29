<script lang="ts">
  import type { Market, Merchant } from '$lib/api';
  import { typeColor, fetchMerchantsByMarket } from '$lib/api';
  import { createEventDispatcher } from 'svelte';

  export let market: Market;

  const dispatch = createEventDispatcher<{ close: undefined }>();

  let merchants: Merchant[] = [];
  let merchantsLoading = false;
  let merchantsError: string | null = null;
  // 무한 스크롤 — 처음 20개 보이고, 스크롤 끝에 가까워지면 20개씩 추가 노출
  const PAGE_SIZE = 20;
  let visibleCount = PAGE_SIZE;
  let listEl: HTMLElement;

  $: if (market) loadMerchants(market);

  async function loadMerchants(m: Market) {
    merchants = [];
    merchantsError = null;
    merchantsLoading = true;
    visibleCount = PAGE_SIZE;
    const hint = m.address_road ?? m.address_jibun ?? '';
    try {
      const res = await fetchMerchantsByMarket(m.name, hint);
      if (market.name !== m.name) return;
      merchants = res;
    } catch (e: any) {
      merchantsError = e.message ?? String(e);
    } finally {
      merchantsLoading = false;
    }
  }

  function onListScroll(e: Event) {
    const el = e.currentTarget as HTMLElement;
    if (el.scrollTop + el.clientHeight >= el.scrollHeight - 80) {
      // 바닥 근처 도달 → 페이지 추가
      if (visibleCount < merchants.length) {
        visibleCount = Math.min(merchants.length, visibleCount + PAGE_SIZE);
      }
    }
  }

  function naverMapUrl(m: Market): string {
    const q = encodeURIComponent(m.address_road ?? m.address_jibun ?? m.name);
    return `https://map.naver.com/p/search/${q}`;
  }
  function kakaoMapUrl(m: Market): string {
    const q = encodeURIComponent(m.address_road ?? m.address_jibun ?? m.name);
    return `https://map.kakao.com/?q=${q}`;
  }

  $: visibleMerchants = merchants.slice(0, visibleCount);
</script>

<div class="backdrop" on:click={() => dispatch('close')} role="presentation"></div>
<div class="card" role="dialog" aria-label="시장 상세">
  <header style="border-left-color: {typeColor(market.market_type)}">
    <div class="title-row">
      <h2>{market.name}</h2>
      <button class="close" on:click={() => dispatch('close')} aria-label="닫기">✕</button>
    </div>
    <div class="meta">
      <span class="type-tag" style="background: {typeColor(market.market_type)}">
        {market.market_type || '시장'}
      </span>
      {#if market.open_cycle}<span>{market.open_cycle}</span>{/if}
      {#if market.store_count != null}<span>{market.store_count}개 점포</span>{/if}
    </div>
  </header>

  <div class="body">
    {#if market.address_road || market.address_jibun}
      <p class="addr">📍 {market.address_road ?? market.address_jibun}</p>
    {/if}

    {#if market.main_items}
      <p class="row"><strong>취급품목</strong> {market.main_items}</p>
    {/if}

    <div class="amenities">
      {#if market.has_parking === 'Y'}<span class="chip">🅿️ 주차장</span>{/if}
      {#if market.has_toilet === 'Y'}<span class="chip">🚻 화장실</span>{/if}
      {#if market.gift_certs}<span class="chip">🎟️ {market.gift_certs}</span>{/if}
    </div>

    <div class="more">
      {#if market.phone}<span>☎ {market.phone}</span>{/if}
      {#if market.open_year}<span>개설 {market.open_year}년</span>{/if}
    </div>

    <div class="actions">
      <a class="btn" href={naverMapUrl(market)} target="_blank" rel="noopener">네이버 지도</a>
      <a class="btn" href={kakaoMapUrl(market)} target="_blank" rel="noopener">카카오맵</a>
      {#if market.homepage}
        <a class="btn outline" href={market.homepage} target="_blank" rel="noopener">홈페이지</a>
      {/if}
    </div>

    <!-- 온누리 가맹점 섹션 -->
    <section class="merchants">
      <h3>
        🎟️ 온누리상품권 가맹점
        {#if !merchantsLoading && !merchantsError}
          <span class="count">{merchants.length}</span>
        {/if}
      </h3>
      {#if merchantsLoading}
        <div class="state">불러오는 중…</div>
      {:else if merchantsError}
        <div class="state err">⚠️ {merchantsError}</div>
      {:else if merchants.length === 0}
        <div class="state empty">이 시장의 온누리 가맹점 정보 없음</div>
      {:else}
        <ul
          class="merchant-list"
          bind:this={listEl}
          on:scroll={onListScroll}
        >
          {#each visibleMerchants as m}
            <li>
              <div class="m-name">{m.name}</div>
              <div class="m-meta">
                {#if m.items}<span>{m.items}</span>{/if}
                <span class="vouchers">
                  {#if m.paper}<span title="지류형">📜</span>{/if}
                  {#if m.digital}<span title="디지털형">📱</span>{/if}
                </span>
              </div>
            </li>
          {/each}
          {#if visibleCount < merchants.length}
            <li class="loading-row">⌛ {merchants.length - visibleCount}개 더…</li>
          {:else if merchants.length > PAGE_SIZE}
            <li class="end-row">— 모두 표시됨 ({merchants.length}개) —</li>
          {/if}
        </ul>
      {/if}
    </section>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 30, 50, 0.45);
    z-index: 1000;
    backdrop-filter: blur(2px);
  }
  .card {
    position: fixed;
    z-index: 1001;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    background: #fff;
    border-radius: 14px;
    padding: 0;
    box-shadow: 0 12px 48px rgba(0, 0, 0, 0.32);
    width: min(380px, 92vw);
    max-height: 88vh;
    font-size: 13px;
    line-height: 1.5;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .body {
    overflow-y: auto;
    padding: 0 16px 14px;
  }
  header {
    border-left: 4px solid #2E7D32;
    padding: 14px 16px 12px 12px;
    margin: 0;
    background: #fafafa;
    border-bottom: 1px solid #f0f0f0;
  }
  .title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  h2 {
    margin: 0 0 4px 0;
    font-size: 17px;
    color: #1B5E20;
  }
  .close {
    background: rgba(0, 0, 0, 0.06);
    border: none;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    cursor: pointer;
    font-size: 14px;
    color: #555;
  }
  .close:hover { background: #d32f2f; color: #fff; }
  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    font-size: 11px;
    color: #555;
  }
  .type-tag {
    color: #fff;
    padding: 2px 8px;
    border-radius: 10px;
    font-weight: 600;
  }
  .addr {
    margin: 10px 0 4px;
    color: #333;
  }
  .row {
    margin: 4px 0;
    color: #333;
  }
  .row strong {
    color: #1B5E20;
    margin-right: 6px;
    font-size: 11px;
    background: #E8F5E9;
    padding: 1px 6px;
    border-radius: 6px;
  }
  .amenities {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin: 6px 0;
  }
  .chip {
    background: #f4f4f4;
    border-radius: 10px;
    padding: 2px 8px;
    font-size: 11px;
    color: #444;
  }
  .more {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    font-size: 11px;
    color: #777;
    margin-top: 6px;
  }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 10px;
  }
  .btn {
    flex: 1;
    background: #2E7D32;
    color: #fff;
    text-decoration: none;
    text-align: center;
    padding: 7px;
    border-radius: 8px;
    font-size: 12px;
    font-weight: 600;
  }
  .btn:hover { filter: brightness(1.08); }
  .btn.outline {
    background: transparent;
    color: #2E7D32;
    border: 1.5px solid #2E7D32;
  }

  /* 가맹점 섹션 */
  .merchants {
    margin-top: 14px;
    padding-top: 12px;
    border-top: 1px dashed #ddd;
  }
  .merchants h3 {
    margin: 0 0 8px 0;
    font-size: 13px;
    color: #1B5E20;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .count {
    background: #2E7D32;
    color: #fff;
    border-radius: 10px;
    padding: 1px 8px;
    font-size: 11px;
    font-weight: 600;
  }
  .state {
    font-size: 12px;
    color: #888;
    padding: 8px 0;
    text-align: center;
  }
  .state.empty { color: #999; font-style: italic; }
  .state.err { color: #d32f2f; }
  .merchant-list {
    list-style: none;
    padding: 0;
    margin: 0;
    max-height: 280px;
    overflow-y: auto;
    border: 1px solid #f0f0f0;
    border-radius: 8px;
  }
  .merchant-list li {
    padding: 6px 10px;
    border-bottom: 1px solid #f5f5f5;
    font-size: 12px;
  }
  .merchant-list li:last-child { border-bottom: none; }
  .m-name {
    font-weight: 600;
    color: #333;
  }
  .m-meta {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: #777;
    margin-top: 1px;
  }
  .vouchers {
    display: inline-flex;
    gap: 2px;
  }
  .loading-row,
  .end-row {
    text-align: center;
    color: #999;
    font-size: 11px;
    background: #fafafa;
    padding: 8px 6px;
  }
  .end-row { color: #bbb; font-style: italic; }
</style>
