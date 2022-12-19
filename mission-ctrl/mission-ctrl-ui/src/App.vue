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
    <div class="row">
      <HardwareDisplay class="columnLeft" :valves="valveDataset"/>
      <RocketTerminal class="columnMiddle" id="terminal"/>
      <SoftwareDisplay class="columnRight" :states="softwareDataset"/>
    </div>
  </div>
</template>

<script>
import RealtimeLineGraph from './components/RealtimeLineGraph.vue';
import RocketTerminal from './components/RocketTerminal.vue';
import HardwareDisplay from './components/HardwareDisplay.vue';
import SoftwareDisplay from './components/SoftwareDisplay.vue';

export default {
  name: 'App',
  components: {
    RealtimeLineGraph,
    RocketTerminal,
    HardwareDisplay,
    SoftwareDisplay
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
        if (Array.isArray(data[key])) {
          data[key] = data[key].filter((_value, index) => index % this.dataDivisor == 0);
        }
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
    valveDataset() {
      return [
        {
          name: 'IG Fuel Valve',
          data: this.dataset.igniter_fuel_valve,
        },
        {
          name: 'IG GOx Valve',
          data: this.dataset.igniter_gox_valve,
        },
        {
          name: 'Fuel Press Valve',
          data: this.dataset.fuel_press_valve,
        },
        {
          name: 'Fuel Vent Valve',
          data: this.dataset.fuel_vent_valve,
        },
        {
          name: 'IG Spark Plug',
          data: this.dataset.sparking,
        },
      ];
    },
    softwareDataset() {
      return [
        {
          name: 'Igniter State',
          value: this.dataset.igniter_state,
        },
        {
          name: 'Fuel Tank State',
          value: this.dataset.tank_state,
        },
        {
          name: 'Telemetry Rate',
          value: this.dataset.telemetry_rate,
          units: "Hz",
        },
        {
          name: 'DAQ Rate',
          value: this.dataset.daq_rate,
          units: "Hz",
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

<style scoped>
#app {
  font-family: Helvetica, Arial;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
}

#terminal {
  height: 25rem;
}

.row {
  display: flex;
  margin-bottom: 20px;
}

.columnLeft {
  flex: 33%;
}

.columnMiddle {
  flex: 33%;
}

.columnRight {
  flex: 33%;
}

.columnRightTwoThirds {
  flex: 67%;
  max-width: 66%;
}

</style>
