<template>
  <div>
    <h2>List of Pods</h2>
    <ul>
      <li v-for="pod in pods" :key="pod.metadata.name">
        {{ pod.metadata.name }}
      </li>
    </ul>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue';
import { Client, config } from '@kubernetes/client-node';

export default {
  name: 'KubernetesPodList',
  setup() {
    const pods = ref([]);

    onMounted(async () => {
      try {
        const kc = new config.KubeConfig();
        kc.loadFromDefault();

        const k8sClient = new Client({ config: kc });

        const response = await k8sClient.api.v1.namespaces('default').pods.get();
        console.log('Response from Kubernetes cluster:', response.body.items);
        pods.value = response.body.items;
      } catch (error) {
        console.error('Error:', error);
      }
    });

    return {
      pods,
    };
  },
};
</script>
