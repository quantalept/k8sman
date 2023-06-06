<template>
  <div>
    <ul>
      <li v-for="pod in pods" :key="pod.name">{{ pod.name }}</li>
    </ul>
  </div>
</template>

<script>
export default {
  data() {
    return {
      pods: [],
    };
  },
  mounted() {
    this.fetchPods();
  },
  methods: {
    async fetchPods() {
      try {
        const response = await window.tauri.promisified({ cmd: 'listPods' });
        this.pods = response.data;
      } catch (error) {
        console.error(error);
      }
    },
  },
};
</script>
