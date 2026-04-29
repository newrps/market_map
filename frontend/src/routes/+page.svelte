<script lang="ts">
  import { onMount } from 'svelte';
  import Map from '$lib/components/Map.svelte';
  import MarketDetail from '$lib/components/MarketDetail.svelte';
  import { fetchMarkets, typeCategory } from '$lib/api';
  import type { Market } from '$lib/api';

  let markets: Market[] = [];
  let selected: Market | null = null;
  let loadError: string | null = null;
  let loading = true;

  onMount(async () => {
    try {
      markets = await fetchMarkets();
    } catch (e: any) {
      loadError = e.message ?? String(e);
    } finally {
      loading = false;
    }
  });

  $: stats = {
    total: markets.length,
    sangseol: markets.filter((m) => typeCategory(m.market_type) === '상설장').length,
    jeonggi: markets.filter((m) => typeCategory(m.market_type) === '정기장').length
  };
</script>

<main class="full-map">
  <Map
    {markets}
    on:select={(e) => (selected = e.detail)}
  />

  <!-- 좌상단 브랜드 + 작은 범례 -->
  <div class="overlay top-left">
    <div class="brand">
      <svg viewBox="0 0 40 40" class="logo-icon" aria-hidden="true">
        <circle cx="20" cy="20" r="19" fill="#2E7D32" />
        <path d="M12 16 H28 V20 H26 V30 H14 V20 H12 Z" fill="#fff" opacity="0.9" />
        <path d="M14 13 Q20 6 26 13 V16 H14 Z" fill="#FFB300" />
      </svg>
      <h1>Ps전통시장지도</h1>
    </div>

    {#if !loading && stats.total > 0}
      <div class="legend">
        <span class="leg"><i style="background:#2E7D32"></i>상설장 {stats.sangseol}</span>
        <span class="leg"><i style="background:#FB8C00"></i>3·4·5일장 {stats.jeonggi}</span>
      </div>
    {/if}
  </div>

  {#if loading}
    <div class="overlay center loading">시장 데이터 불러오는 중...</div>
  {/if}

  {#if loadError}
    <div class="overlay bottom-center error-banner">
      ⚠️ 데이터 로드 실패: {loadError}
    </div>
  {/if}


  {#if selected}
    <MarketDetail market={selected} on:close={() => (selected = null)} />
  {/if}
</main>

<style>
  .full-map {
    position: fixed;
    inset: 0;
    overflow: hidden;
  }
  :global(.full-map > .map) {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }

  .overlay {
    position: absolute;
    z-index: 500;
    pointer-events: none;
  }
  .overlay > * { pointer-events: auto; }

  .top-left {
    top: 16px;
    left: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    max-width: calc(100vw - 32px);
  }

  .brand {
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(10px);
    border-radius: 12px;
    padding: 8px 14px;
    display: inline-flex;
    align-items: center;
    gap: 10px;
    box-shadow: 0 2px 12px rgba(0, 61, 92, 0.15);
    width: fit-content;
  }
  .logo-icon { width: 32px; height: 32px; }
  .brand h1 {
    margin: 0;
    font-size: 18px;
    color: #1B5E20;
    font-weight: 700;
    letter-spacing: -0.5px;
  }

  .legend {
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(10px);
    border-radius: 12px;
    padding: 6px 12px;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
    display: inline-flex;
    align-items: center;
    gap: 12px;
    width: fit-content;
    font-size: 12px;
    color: #333;
  }
  .leg {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }
  .leg i {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    display: inline-block;
  }

  .center {
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
  }
  .loading {
    background: rgba(255, 255, 255, 0.9);
    padding: 12px 18px;
    border-radius: 10px;
    color: #1B5E20;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
  }

  .bottom-center {
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
  }
  .error-banner {
    background: #ffebee;
    color: #b71c1c;
    border-left: 3px solid #d32f2f;
    padding: 10px 16px;
    border-radius: 8px;
    font-size: 13px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  @media (max-width: 480px) {
    .top-left {
      top: 8px;
      left: 8px;
      right: 8px;
    }
    .brand h1 { font-size: 16px; }
    .legend { font-size: 11px; gap: 8px; }
  }
</style>
