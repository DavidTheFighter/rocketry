<template>
  <div>
    <div class="row">
      <RealtimeLineGraph 
      :datasets="igniterDataset" 
      :yrange="[0, 250]"
      :xTitle="'Time (sec)'"
      :yTitle="'Pressure (PSI)'"
      :paddingFigs="3"
      class="columnLeft"
      />
      <RealtimeLineGraph 
      :datasets="tankDataset" 
      :yrange="[0, 250]" 
      :xTitle="'Time (sec)'"
      :yTitle="'Pressure (PSI)'"
      :paddingFigs="3"
      :justifyLegend="'left'"
      class="columnMiddle"
      />
      <RealtimeLineGraph 
      :datasets="tempDataset" 
      :xTitle="'Time (sec)'"
      :yTitle="'Temperature (°C)'"
      :paddingFigs="3"
      :justifyLegend="'left'"
      class="columnLeft"
      />
    </div>
  </div>
</template>

<script>
import RealtimeLineGraph from './components/RealtimeLineGraph.vue'

export default {
  name: 'App',
  components: {
    RealtimeLineGraph,
  },
  props: {
    refreshTimeMillis: {
      type: Number,
      default: 33
    },
    dataDivisor: {
      type: Number,
      default: 3
    },
  },
  watch: {
    timer: {
      async handler() {
        this.generateData();

        setTimeout(() => {
          this.timer += 1;
        }, this.refreshTimeMillis);
      },
      immediate: true,
    }
  },
  methods: {
    async generateData() {
      const response = await fetch('http://localhost:8000/telemetry');
      const data = await response.json();

      for (const key in data) {
        data[key] = data[key].filter((_value, index) => index % this.dataDivisor == 0);
      }

      this.dataset = data;
    }
  },
  computed: {
    igniterDataset() {
      return [
        {
          name: 'IG GOx',
          color: 'cyan',
          data: this.dataset.igniter_gox_pressure,
          units: "PSI",
        },
        {
          name: 'IG Fuel',
          color: 'orange',
          data: this.dataset.igniter_fuel_pressure,
          units: "PSI",
        },
        {
          name: 'IG Chamber',
          color: 'red',
          data: this.dataset.igniter_chamber_pressure,
          units: "PSI",
        },
      ];
    },
    tankDataset() {
      return [
        {
          name: 'Fuel Tank',
          color: 'green',
          data: this.dataset.fuel_tank_pressure,
          units: "PSI",
        },
      ];
    },
    tempDataset() {
      return [
        {
          name: 'ECU Board',
          color: '#7FFFD4',
          data: this.dataset.ecu_board_temp,
          units: "°C",
        },
        {
          name: 'IG Throat',
          color: '#A9A9A9',
          data: this.dataset.igniter_throat_temp,
          units: "°C",
        },
      ];
    },
  },
  data() {
    return {
      timer: 0,
      dataset: [],
    }
  }
}
</script>

<style>
#app {
  font-family: Helvetica, Arial;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
}

.row {
  display: flex;
}

.columnLeft {
  flex: 33%;
  max-width: 32%;
  margin-right: auto;
}

.columnMiddle {
  flex: 33%;
  max-width: 32%;
  margin-right: auto;
}

.columnRight {
  flex: 33%;
  max-width: 32%;
  margin-left: auto;
}

</style>
