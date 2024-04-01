<template>
  <div>
    <div class="row">
      <RealtimeLineGraphChartjs
        :data-description="igniterDataset"
        :dataset="graph_data"
        :yrange="[0, 250]"
        :xTitle="'Time (sec)'"
        :yTitle="'Pressure (PSI)'"
        :displayTimeSeconds="20.0"
        :displayTickInterval="2.0"
        :paddingFigs="3"
        class="columnLeft"
      />
      <RealtimeLineGraphChartjs
        :data-description="tankDataset"
        :dataset="graph_data"
        :yrange="[0, 300]"
        :xTitle="'Time (sec)'"
        :yTitle="'Pressure (PSI)'"
        :displayTimeSeconds="20.0"
        :displayTickInterval="2.0"
        :paddingFigs="3"
        :justifyLegend="'left'"
        class="columnMiddle"
      />
      <RealtimeLineGraphChartjs
        :data-description="tempDataset"
        :dataset="graph_data"
        :xTitle="'Time (sec)'"
        :yTitle="'Temperature (°C)'"
        :displayTimeSeconds="20.0"
        :displayTickInterval="2.0"
        :paddingFigs="3"
        :justifyLegend="'left'"
        class="columnLeft"
      />
    </div>
    <div class="row">
      <HardwareDisplay class="columnLeft" :valves="valveDataset"/>
      <RocketTerminal class="columnMiddle" id="terminal"/>
      <DatasetDisplay class="columnRight" :states="softwareDataset"/>
    </div>
  </div>
</template>

<script>
import RealtimeLineGraphChartjs from '../components/RealtimeLineGraphChartjs.vue';
import RocketTerminal from '../components/RocketTerminal.vue';
import HardwareDisplay from '../components/HardwareDisplay.vue';
import DatasetDisplay from '../components/DatasetDisplay.vue';
import * as util from '../util/data.js';

export default {
  name: 'IgniterPage',
  components: {
    RealtimeLineGraphChartjs,
    RocketTerminal,
    HardwareDisplay,
    DatasetDisplay,
  },
  props: {
    refreshTimeMillis: {
      type: Number,
      default: 33
    },
  },
  computed: {
    igniterDataset() {
      return [
        {
          name: 'IG GOx',
          color: 'cyan',
          dataName: 'igniter_oxidizer_pressure_psi',
          units: "PSI",
        },
        {
          name: 'IG Fuel',
          color: 'orange',
          dataName: 'igniter_fuel_pressure_psi',
          units: "PSI",
        },
        {
          name: 'IG Chamber',
          color: 'red',
          dataName: 'igniter_chamber_pressure_psi',
          units: "PSI",
        },
      ];
    },
    tankDataset() {
      return [
        {
          name: 'Fuel Tank',
          color: 'orange',
          dataName: "fuel_tank_pressure_psi",
          units: "PSI",
        },
        {
          name: 'Oxidizer Tank',
          color: 'cyan',
          dataName: "oxidizer_tank_pressure_psi",
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
        {
          name: 'CPU Usage',
          value: this.dataset.cpu_utilization,
          units: "%",
        },
      ];
    },
  },
  watch: {
    timer: {
      async handler() {
        this.generateData();
      },
      immediate: true,
    }
  },
  methods: {
    async generateData() {
      let debug_data = undefined;

      try {
        debug_data = await this.fetcher.fetch('http://localhost:8000/ecu-telemetry/0/debug-data');
      } catch (error) {
        console.log(error);
      }

      try {
        let dataset = await this.fetcher.fetch('http://localhost:8000/ecu-telemetry/0');
        dataset.debug_data = debug_data;

        this.dataset = dataset;
      } catch (error) {
        console.log(error);
      }

      try {
        this.graph_data = await this.fetcher.fetch('http://localhost:8000/ecu-telemetry/0/graph');
      } catch (error) {
        console.log(error);
      }

      this.timer += 1;
    },
  },
  created() {
    this.fetcher = new util.DataFetcher(this.refreshTimeMillis - 1);
  },
  data() {
    return {
      timer: 0,
      dataset: {},
      graph_data: {},
    }
  },
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
  max-height: 100%;
  padding-left: 20px;
}

.columnRightTwoThirds {
  flex: 67%;
  max-width: 66%;
}

</style>
