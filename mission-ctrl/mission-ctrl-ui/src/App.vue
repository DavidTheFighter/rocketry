<script>
import FcuDebugPage from './pages/FcuDebugPage.vue';
import IgniterPage from './pages/IgniterPage.vue';
import NotFound from './pages/NotFound.vue';
import RocketPage from './pages/RocketPage.vue';
import RocketSimPage from './pages/RocketSimPage.vue';
import StreamTestPage from './pages/StreamTestPage.vue';

const routes = {
  '/': IgniterPage,
  '/fcudebug': FcuDebugPage,
  '/notfound': NotFound,
  '/igniter': IgniterPage,
  '/rocket': RocketPage,
  '/rocketsim': RocketSimPage,
  '/streamtest': StreamTestPage,
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

<style>
body {
	background-color: #222;
	color: #ddd;
}

a[href] {
	color: #1e8ad6;
}

a[href]:hover {
	color: #3ba0e6;
}

aside,
[class *= "sidebar"],
[id *= "sidebar"] {
	max-width: 90%;
	margin: 0 auto;
	border: 1px solid lightgrey;
	padding: 5px 15px;
}

</style>