<template>
  <p>All of the pages available:</p>
  <div v-for="route in allRoutes" :key="route.name">
    <p>- {{ route.name }}</p>
  </div>
</template>

<script>
import App from '../App.vue';

export default {
  name: 'HomePage',
  computed: {
    allRoutes() {
      return App.routes;
    },
  },
}
</script>
