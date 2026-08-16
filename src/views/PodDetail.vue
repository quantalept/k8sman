<script setup lang="ts">
import { useRoute } from "vue-router";
import { NCard, NText } from "naive-ui";

const route = useRoute();
</script>

<template>
  <n-card :title="`Pod: ${route.params.name}`">
    <n-text depth="3">
      Logs, exec, port-forward, and file copy for this pod land in their respective
      build-order steps.
    </n-text>
  </n-card>
</template>
