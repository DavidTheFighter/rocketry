<script>
import IgniterPage from './pages/IgniterPage.vue'
import NotFound from './pages/NotFound.vue'
import RocketPage from './pages/RocketPage.vue'

const routes = {
  '/': IgniterPage,
  '/igniter': IgniterPage,
  '/rocket': RocketPage,
  '/notfound': NotFound,
};

export default {
  data() {
    return {
      currentPath: window.location.pathname,
    }
  },
  computed: {
    currentView() {
      return routes[this.currentPath] || NotFound;
    }
  },
  mounted() {
    window.addEventListener('hashchange', () => {
      this.currentPath = window.location.pathname;
		})
  }
}
</script>

<template>
  <component :is="currentView" />
</template>
