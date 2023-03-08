<script>
import { defineComponent } from 'vue';
import Homepage from "./Homepage.vue";

export default defineComponent({
  name: 'K8Sman',
  components: {
    Homepage,
  },
});
</script>
<template>
  <v-app>
    <header>
      <v-app-bar app>
        <v-toolbar-title>K8sman</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn icon>
          <v-icon>mdi-account-circle-outline</v-icon>
        </v-btn>
        <v-btn color="primary">Click me</v-btn>
      </v-app-bar>
    </header>
    <Homepage></Homepage>
  </v-app>
</template>