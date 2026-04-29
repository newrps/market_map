<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import type { Market } from '$lib/api';
  import { typeColor } from '$lib/api';

  export let markets: Market[] = [];

  const dispatch = createEventDispatcher<{ select: Market }>();

  let mapEl: HTMLDivElement;
  let map: any;
  let L: any;
  let markersLayer: any;
  let popup: any;

  onMount(async () => {
    L = (await import('leaflet')).default;
    await import('leaflet/dist/leaflet.css');

    map = L.map(mapEl, {
      center: [36.5, 127.8],
      zoom: 7,
      zoomControl: true,
      attributionControl: true
    });

    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      attribution: '© OpenStreetMap',
      subdomains: 'abc',
      maxZoom: 19
    }).addTo(map);

    markersLayer = L.layerGroup().addTo(map);
    renderMarkers();
  });

  onDestroy(() => {
    if (map) map.remove();
  });

  function renderMarkers() {
    if (!map || !L) return;
    markersLayer.clearLayers();

    for (const m of markets) {
      if (m.lat == null || m.lon == null) continue;
      if (Math.abs(m.lat) < 1 || Math.abs(m.lon) < 1) continue; // 좌표 0/null 가드

      const marker = L.circleMarker([m.lat, m.lon], {
        radius: 6,
        fillColor: typeColor(m.market_type),
        color: '#fff',
        weight: 1.5,
        fillOpacity: 0.9
      });

      marker.bindTooltip(m.name, { direction: 'top', offset: [0, -6] });
      marker.on('click', () => dispatch('select', m));
      marker.addTo(markersLayer);
    }
  }

  $: if (map && markets) renderMarkers();
</script>

<div class="map" bind:this={mapEl}></div>

<style>
  .map {
    width: 100%;
    height: 100%;
    background: #cfe5f3;
  }
</style>
