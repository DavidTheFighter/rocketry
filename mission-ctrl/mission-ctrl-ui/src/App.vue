<script>
import HomePage from './pages/HomePage.vue';
import EcuDebugPage from './pages/EcuDebugPage.vue';
import FcuDebugPage from './pages/FcuDebugPage.vue';
import IgniterPage from './pages/IgniterPage.vue';
import NotFound from './pages/NotFound.vue';
import RocketPage from './pages/RocketPage.vue';
import RocketSimPage from './pages/RocketSimPage.vue';
import StreamTestPage from './pages/StreamTestPage.vue';
import PumpTestPage from './pages/PumpTestPage.vue';
import EngineColdFlowPage from './pages/EngineColdFlowPage.vue';

export const routes = {
  '/': HomePage,
  '/fcudebug': FcuDebugPage,
  '/ecudebug': EcuDebugPage,
  '/notfound': NotFound,
  '/igniter': IgniterPage,
  '/rocket': RocketPage,
  '/rocketsim': RocketSimPage,
  '/streamtest': StreamTestPage,
  '/pumptest': PumpTestPage,
  '/enginecoldflow': EngineColdFlowPage,
};

export default {
  routes: routes,
  data() {
    return {
      currentPath: window.location.pathname,
      routes: routes,
    }
  },
  computed: {
    currentView() {
      return this.routes[this.currentPath] || NotFound;
    },
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