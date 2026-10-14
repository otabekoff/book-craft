<script setup lang="ts">
defineProps<{ name: string }>();

const paths: Record<string, string> = {
  write: "M4 20h4L19 9l-4-4L4 16v4zM14 6l4 4",
  details: "M6 3h9l4 4v14H6zM14 3v5h5M9 12h7M9 16h7",
  appearance: "M12 3a9 9 0 1 0 0 18c1.1 0 1.6-.8 1.6-1.6 0-1.2-1-1.4-1-2.4 0-.9.7-1.6 1.6-1.6H17a4 4 0 0 0 4-4c0-4.7-4-8.4-9-8.4zM7.5 11.5h.01M10 7.5h.01M15 7.5h.01",
  modules: "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
  assets: "M4 5h16v14H4zM4 15l5-5 4 4 3-3 4 4M15 9h.01",
  build: "M12 3v12M7 10l5 5 5-5M5 19h14",
  plus: "M12 5v14M5 12h14",
  more: "M5 12h.01M12 12h.01M19 12h.01",
  folder: "M3 6h6l2 2h10v11H3z",
  close: "M6 6l12 12M18 6 6 18",
  eye: "M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12zM12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
  book: "M4 19V6a2 2 0 0 1 2-2h12v13H6a2 2 0 0 0-2 2zm0 0a2 2 0 0 0 2 2h12",
  play: "M7 4l12 8-12 8z",
  sun: "M12 4V2M12 22v-2M4 12H2M22 12h-2M5.6 5.6 4.2 4.2M19.8 19.8l-1.4-1.4M5.6 18.4l-1.4 1.4M19.8 4.2l-1.4 1.4M12 17a5 5 0 1 0 0-10 5 5 0 0 0 0 10z",
  moon: "M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z",
  sepia: "M4 4h16v16H4z",
  split: "M4 4h16v16H4zM12 4v16",
  bold: "M7 4h6a4 4 0 0 1 0 8H7zM7 12h7a4 4 0 0 1 0 8H7z",
  italic: "M10 4h8M6 20h8M14 4l-4 16",
  strike: "M5 12h14M16 6a4 3 0 0 0-8 1c0 4 8 3 8 7a4 3 0 0 1-8 1",
  code: "M8 8l-4 4 4 4M16 8l4 4-4 4",
  link: "M10 14a4 4 0 0 0 6 0l3-3a4 4 0 0 0-6-6l-1 1M14 10a4 4 0 0 0-6 0l-3 3a4 4 0 0 0 6 6l1-1",
  image: "M4 5h16v14H4zM4 15l5-5 4 4 3-3 4 4",
  quote: "M7 7h4v4c0 3-2 5-4 6M15 7h4v4c0 3-2 5-4 6",
  list: "M9 6h11M9 12h11M9 18h11M4 6h.01M4 12h.01M4 18h.01",
  olist: "M10 6h10M10 12h10M10 18h10M4 4v4M3 12h2l-2 3h2",
  h: "M5 4v16M15 4v16M5 12h10M19 12v8",
  mark: "M4 20h6M14 4l6 6-9 9H5v-6z",
  table: "M3 5h18v14H3zM3 10h18M3 15h18M9 5v14M15 5v14",
  check: "M5 12l5 5 9-10",
  warn: "M12 3 2 20h20zM12 10v4M12 17h.01",
  trash: "M4 7h16M10 11v6M14 11v6M6 7l1 13h10l1-13M9 7V4h6v3",
  up: "M12 19V5M6 11l6-6 6 6",
  down: "M12 5v14M6 13l6 6 6-6",
  search: "M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14zM21 21l-5-5",
};
</script>

<template>
  <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path :d="paths[name] || paths.more" /></svg>
</template>
