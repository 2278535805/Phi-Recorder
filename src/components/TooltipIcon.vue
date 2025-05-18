<script setup lang="ts">
defineProps<{ tooltip: string; btnClass?: string[] | string }>();
</script>

<template>
  <v-tooltip location="bottom" max-width="250px">
    <template v-slot:activator="{ props }">
      <v-btn :class="btnClass" icon v-bind="props" :ripple="false" variant="plain">
        <v-icon> mdi-information-outline </v-icon>
      </v-btn>
    </template>
    <span style="white-space: pre-wrap;">{{ tooltip }}</span>
  </v-tooltip>
</template>
