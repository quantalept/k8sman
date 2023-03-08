<template>
    <div class="home-page">
      <v-container>
        <v-card class="mb-5">
          <v-card-title>
            Welcome to K8sman
          </v-card-title>
          
        </v-card>
      </v-container>
    </div>
  </template>
  
  <script>
  import { defineComponent } from 'vue';
  
  export default defineComponent({
    name: "HomePage",
  });
  </script>
  
  <style scoped>
  .home-page {
    padding: 80px;
  }
  
  .v-card-title {
    font-size: 24px;
  }
  
  </style>
  