<template>
  <v-container>
    <v-col md="4" sm="8">
      <v-card class="custom-card" width="550px">
        <v-toolbar color="grey" height="25">
          <v-toolbar-title>Add a new cluster</v-toolbar-title></v-toolbar>

        <v-card-text>
          <v-text-field v-model="value" label="Paste your cluster config"></v-text-field>

          <v-card-actions>

            <v-spacer />
            <v-btn color="#5865f2" variant="tonal" @click="importOrBrowse">Import </v-btn><v-spacer />
          </v-card-actions></v-card-text>
      </v-card></v-col>
  </v-container>
</template>

<script>
export default {
  data() {
    return {
      value: ''
    }
  },
  methods: {
    importOrBrowse() {
      if (this.value.startsWith('http')) {
        // handle importing link logic
      } else {
        // handle browsing files logic
      }
    }
  }
}
</script>

<style scoped>
.custom-card {
  background-color: white;
}

.v-container {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 90vh;

}

.v-text-field {
  max-width: 5500px;
  height: 200px;
}

.v-toolbar-title {
  font-size: 16px;
}</style>
