<script lang="ts">
  import { onMount } from 'svelte';
  import Map from '$lib/components/Map.svelte';
  import MarketDetail from '$lib/components/MarketDetail.svelte';
  import { fetchMarkets, typeColor } from '$lib/api';
  import type { Market } from '$lib/api';

  let markets: Market[] = [];
  let selected: Market | null = null;
  let loadError: string | null = null;
  let loading = true;

  // 시장유형 필터 — 빈 배열이면 전체
  let activeTypes: string[] = [];
  let allTypes: string[] = [];

  onMount(async () => {
    try {
      markets = await fetchMarkets();
      const set = new Set<string>();
      for (const m of markets) {
        if (m.market_type) set.add(m.market_type);
      }
      allTypes = [...set].sort();
    } catch (e: any) {
      loadError = e.message ?? String(e);
    } finally {
      loading = false;
    }
  });

  function toggleType(t: string) {
    activeTypes = activeTypes.includes(t)
      ? activeTypes.filter((x) => x !== t)
      : [...activeTypes, t];
  }

  $: filteredCount = activeTypes.length === 0
    ? markets.length
    : markets.filter((m) => activeTypes.some((t) => m.market_type.includes(t))).length;
</script>

<main class="full-map">
  <Map
    {markets}
    highlightTypes={activeTypes}
    on:select={(e) => (selected = e.detail)}
  />

  <!-- 좌상단 브랜드 + 필터 -->
  <div class="overlay top-left">
    <div class="brand">
      <svg viewBox="0 0 40 40" class="logo-icon" aria-hidden="true">
        <circle cx="20" cy="20" r="19" fill="#2E7D32" />
        <path d="M12 16 H28 V20 H26 V30 H14 V20 H12 Z" fill="#fff" opacity="0.9" />
        <path d="M14 13 Q20 6 26 13 V16 H14 Z" fill="#FFB300" />
      </svg>
      <h1>Ps전통시장지도</h1>
    </div>

    {#if !loading && allTypes.length > 0}
      <div class="filter">
        <button
          class="chip"
          class:active={activeTypes.length === 0}
          on:click={() => (activeTypes = [])}
        >전체 {markets.length}</button>
        {#each allTypes as t}
          <button
            class="chip"
            class:active={activeTypes.includes(t)}
            style="--c: {typeColor(t)}"
            on:click={() => toggleType(t)}
          >
            <span class="dot" style="background: {typeColor(t)}"></span>
            {t}
          </button>
        {/each}
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

  {#if !loading && !loadError && filteredCount > 0}
    <div class="overlay bottom-right hint">
      {filteredCount}개 시장 표시 중
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

  .filter {
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(10px);
    border-radius: 14px;
    padding: 6px;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    max-width: 100%;
    overflow-x: auto;
  }
  .chip {
    background: transparent;
    border: 1.5px solid transparent;
    border-radius: 12px;
    padding: 4px 10px;
    font-size: 12px;
    color: #1B5E20;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: inherit;
    font-weight: 500;
    white-space: nowrap;
    transition: all 0.12s;
  }
  .chip:hover { background: rgba(46, 125, 50, 0.08); }
  .chip.active {
    background: rgba(46, 125, 50, 0.12);
    border-color: var(--c, #2E7D32);
    font-weight: 600;
  }
  .chip .dot {
    width: 8px;
    height: 8px;
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
  .bottom-right {
    bottom: 16px;
    right: 16px;
  }
  .hint {
    background: rgba(46, 125, 50, 0.85);
    color: #fff;
    padding: 6px 12px;
    border-radius: 14px;
    font-size: 11px;
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
    .filter { font-size: 11px; }
  }
</style>
