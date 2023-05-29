<template>
  <div>
    <ul>
      <li v-for="pod in pods" :key="pod.metadata.name">{{ pod.metadata.name }}</li>
    </ul>
  </div>
</template>

<script>
import { listPods } from '../k8s/kubernetes';

export default {
  data() {
    return {
      pods: []
    };
  },
  mounted() {
    this.listPods();
  },
  methods: {
    async listPods() {
      console.log('Fetching pods...');
      this.pods = await fetchPods();
      console.log('Fetched pods:', this.pods);
    }
  }
};
</script>
