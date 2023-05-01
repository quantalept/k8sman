<!-- 
<template>
  <v-app>
    <app-bar></app-bar>
    <v-navigation-drawer width="48" permanent color="#E5E4E2" location="right">
      <v-btn variant="plain" class="text-center" size="51">
        <v-icon>mdi-axis-x-arrow</v-icon>
      </v-btn>
    </v-navigation-drawer>
    <app-main></app-main>
    <v-content>
    <homepage class="sum"></homepage>
    </v-content>
  </v-app>
</template>

<script>
import Menubar from './Components/Menubar.vue'
import Main from './Components/Main.vue'
import Homepage from './Homepage.vue'

export default {
  components: {
    'app-bar': Menubar,
    'app-main': Main,
    Homepage
  },
  data() {
    return {
      loginDialog: false,
    }
  },
}
</script>
<style scoped>

</style> -->
<template>
  <v-app>
    <app-bar></app-bar>
    <v-navigation-drawer width="48" permanent color="#E5E4E2" location="right">
      <v-btn variant="plain" class="text-center" size="51">
        <v-icon>mdi-axis-x-arrow</v-icon>
      </v-btn>
    </v-navigation-drawer>

    <app-main></app-main>

  </v-app>
</template>

<script>
import Menubar from './Components/Menubar.vue'
import Main from './Components/Main.vue'
import Homepage from './Components/Homepage.vue'

export default {
  components: {
    'app-bar': Menubar,
    'app-main':  Main,
    'homepage': Homepage
  },
  data() {
    return {
      loginDialog: false,
    }
  },
}
</script>

<style scoped></style>


