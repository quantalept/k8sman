   <template>
    <v-navigation-drawer color="grey-lighten-3" rail>
           <v-avatar
           class="d-block text-center mx-auto mt-4"
             color="grey-darken-1"
             size="36">
            </v-avatar>
           <v-divider class="mx-3 my-5"></v-divider>
           <v-avatar
             v-for="n in 13"
             :key="n"
             class="d-block text-center mx-auto mb-9"
             color="grey-lighten-1"
             size="28"
           ></v-avatar>
         </v-navigation-drawer>
    <v-system-bar window>
    <v-row no-gutters>
      <v-col cols="auto">
        <v-toolbar-title> <v-icon>mdi-account</v-icon> K8s</v-toolbar-title>
      </v-col>
      <v-col>
        <v-row no-gutters class="ml-16" >
          <v-btn color="black" variant="plain" >File</v-btn>
          <v-btn color="black" variant="plain" >View</v-btn>
          <v-btn color="black" variant="plain">Edit</v-btn>
          <v-btn color="black" variant="plain" >Help</v-btn>

        </v-row>
      </v-col>
    </v-row>
  </v-system-bar>

   </template>
   <script>
   export default {
  name: 'AppBar',
}
  </script>