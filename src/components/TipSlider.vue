<script setup lang="ts">
import TooltipIcon from './TooltipIcon.vue';

defineProps<{ tooltip: string }>();
</script>

<template>
  <v-slider>
    <template v-slot:append>
      <TooltipIcon :tooltip="tooltip" btn-class="ml-n2 whitespace-pre"></TooltipIcon>
    </template>
  </v-slider>
</template>
