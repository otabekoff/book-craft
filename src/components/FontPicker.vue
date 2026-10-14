<script setup lang="ts">
// Font chooser for one typography role. Options are shown in their own face;
// bundled fonts say how much they add to the book.
import { computed, onMounted } from "vue";
import { useFontPreviews } from "../fonts";
import { store } from "../store";
import { UiSelect, type SelectOption } from "../ui";

const props = defineProps<{
  /** Font categories to offer (serif, sans, mono, display, handwriting). */
  categories: string[];
  /** Extra first option, e.g. "Same as body". */
  extra?: SelectOption<string>;
  id?: string;
}>();
const model = defineModel<string>({ required: true });

const options = computed<SelectOption<string>[]>(() => {
  const fonts = (store.info?.fonts ?? []).filter((f) => props.categories.includes(f.category) && f.key !== "custom");
  const out: SelectOption<string>[] = props.extra ? [props.extra] : [];
  for (const source of ["library", "windows"] as const) {
    for (const f of fonts.filter((x) => x.source === source)) {
      out.push({
        value: f.key,
        label: f.name,
        description: f.cyrillic ? f.note : `${f.note} · Latin only`,
        group: source === "library" ? "Bundled with the book" : "Windows fonts",
        fontFamily: f.stack,
        meta: f.kb ? `+${f.kb} KB` : undefined,
      });
    }
  }
  if (props.categories.includes("serif")) {
    out.push({ value: "custom", label: "Custom font file…", description: "A font from this book’s Assets", group: "Other" });
  }
  if (model.value && !out.some((o) => o.value === model.value)) out.push({ value: model.value, label: model.value });
  return out;
});

onMounted(useFontPreviews);
</script>

<template>
  <UiSelect :id="id" v-model="model" :options="options" searchable />
</template>
