<template>
  <div>
    <div class="rowThird">
      <DatasetDisplay class="columnThird" :states="softwareDataset"/>
      <VideoStream class="columnThird"/>
      <EmptyComponent class="columnThird"/>
    </div>
    <div class="rowThird">
      <VideoStream class="columnThird"/>
      <RocketTerminal class="columnThird"/>
      <VideoStream class="columnThird"/>
    </div>
    <div class="rowThird">
      <VideoStream class="columnThird"/>
      <VideoStream class="columnThird"/>
      <VideoStream class="columnThird"/>
    </div>
  </div>
</template>

<script>
import DatasetDisplay from '../components/DatasetDisplay.vue';
import VideoStream from '../components/VideoStream.vue';
import EmptyComponent from '../components/EmptyComponent.vue';
import RocketTerminal from '../components/RocketTerminal.vue';

export default {
  name: 'StreamTestPage',
  components: {
    DatasetDisplay,
    VideoStream,
    EmptyComponent,
    RocketTerminal,
  },
  props: {
  },
  computed: {
    softwareDataset() {
      return [
        {
          name: 'Camera State',
          value: "This camera",
        },
        {
          name: 'Value #2',
          value: "42",
        },
        {
          name: 'Telemetry Rate',
          value: 42.69,
          units: "Hz",
        },
      ];
    },
  },
  data() {
    return {
    }
  }
}
</script>

<style scoped>
#app {
  font-family: Helvetica, Arial;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
}

.rowThird {
  display: flex;
  margin-bottom: 5px;
  height: 33vh;
}

.columnThird {
  flex: 33%;
  padding-left: 3px;
  padding-right: 3px;
}

</style>
