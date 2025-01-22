<script setup lang="ts">
import TooltipIcon from './TooltipIcon.vue';

defineProps<{ label: string; tooltip?: string }>();
</script>

<template>
  <v-switch color="primary" class="mx-2 d-flex justify-center whitespace-pre">
    <template v-slot:label>
      {{ label }}
      <TooltipIcon v-if="tooltip" :tooltip="tooltip"></TooltipIcon>
    </template>
  </v-switch>
</template>
