<template>
  <v-navigation-drawer width="240" permanent >
      <v-avatar  color="grey-darken-1" size="35"></v-avatar>
    <v-list dense>
      <v-list-item @click="goToclick" prepend-icon="mdi-kubernetes" title="Cluster"></v-list-item>
      <v-list-item @click="goToclick" prepend-icon="mdi-resistor-nodes" title="Nodes"></v-list-item>
      <v-list-group fluid>
        <template v-slot:activator="{ props }">
          <v-list-item v-bind="props" prepend-icon="mdi-sitemap-outline" title="Workloads"></v-list-item>
        </template>
        <v-list-item v-for="([title], i) in workloadlist" :key="i"  :title="title"></v-list-item>
      </v-list-group>
      <v-list-group fluid>
        <template v-slot:activator="{ props }">
          <v-list-item v-bind="props" prepend-icon="mdi-format-list-bulleted" title="Config"></v-list-item>
        </template>
        <v-list-item v-for="([title], i) in configlist" :key="i"  :title="title"></v-list-item>
      </v-list-group>
      <v-list-group fluid>
        <template v-slot:activator="{ props }">
          <v-list-item v-bind="props" prepend-icon="mdi-lan" title="Network"></v-list-item>
        </template>
        <v-list-item v-for="([title], i) in networklist" :key="i"  :title="title"></v-list-item>
      </v-list-group>
      <v-list-group fluid>
        <template v-slot:activator="{ props }">
          <v-list-item v-bind="props" prepend-icon="mdi-sd" title="Storage"></v-list-item>
        </template>
        <v-list-item v-for="([title], i) in storagelist" :key="i"  :title="title"></v-list-item>
      </v-list-group>
      <v-list-item @click="goToclick" prepend-icon="mdi-collapse-all-outline" title="Namespaces"></v-list-item>
      <v-list-item @click="goToclick" prepend-icon="mdi-clock-time-eight-outline" title="Events"></v-list-item>
      <v-list-group fluid>
        <template v-slot:activator="{ props }">
          <v-list-item v-bind="props" prepend-icon="mdi-ship-wheel" title="Helm"></v-list-item>
        </template>
        <v-list-item v-for="([title], i) in helmlist" :key="i" :value="title" :title="title"></v-list-item>
      </v-list-group>
      <v-list-group fluid>
        <template v-slot:activator="{ props }">
          <v-list-item v-bind="props" prepend-icon="mdi-shield-lock" title="Access Control"></v-list-item>
        </template>
        <v-list-item v-for="([title], i) in accesscontrollist" :key="i"  :title="title"></v-list-item>
      </v-list-group>
      <v-list-group fluid>
        <template v-slot:activator="{ props }">
          <v-list-item  v-bind="props"  prepend-icon="mdi-puzzle"  title="Customer Resources"></v-list-item>
        </template>
        <v-list-item @click="goToclick" title="Definitions"></v-list-item>
        <v-list-group fluid>
          <template v-slot:activator="{ props }">
            <v-list-item  v-bind="props" title="helm.cattle.io"  ></v-list-item>
          </template>
          <v-list-item
            v-for="([title], i) in helmcattleiolist" :key="i" :title="title"  ></v-list-item>
        </v-list-group>
        <v-list-group fluid>
          <template v-slot:activator="{ props }">
            <v-list-item v-bind="props"  title="k3s.cattle.io"></v-list-item>
          </template>
          <v-list-item
            v-for="([title], i) in hk3scattleiolist" :key="i" :title="title" 
          ></v-list-item>
        </v-list-group>
        <v-list-group fluid>
          <template v-slot:activator="{ props }">
            <v-list-item v-bind="props"  title="traefik.containo.us"></v-list-item>
          </template>
          <v-list-item
            v-for="([title], i) in traefikcontainouslist" :key="i" :title="title" 
          ></v-list-item>
        </v-list-group>
      </v-list-group>

      
    </v-list>
  </v-navigation-drawer>
</template>

<script>
export default {
  data: () => ({

  }),

   methods: {
    goToclick() {

     }, 
   },

   computed:{
       workloadlist() {
        return this.$store.getters['Grplist/workloaditems'];
       },
       configlist() {
        return this.$store.getters['Grplist/configitems'];
       },
       networklist() {
        return this.$store.getters['Grplist/networkitems'];
       },
       storagelist() {
        return this.$store.getters['Grplist/storageitems'];
       },
       helmlist() {
        return this.$store.getters['Grplist/helmitems']
       },
       accesscontrollist() {
        return this.$store.getters['Grplist/accesscontrolitems']
       },
       helmcattleiolist() {
        return this.$store.getters['Grplist/helmcattleioitems'];
       },
       hk3scattleiolist() {
        return this.$store.getters['Grplist/hk3scattleioitems']
       },
       traefikcontainouslist() {
        return this.$store.getters['Grplist/traefikcontainousitems']
       }
  },
}
</script>
<style>

</style>