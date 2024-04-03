<template>
  <div>
    <div class="row">
      <RealtimeLineGraphChartjs
        :data-description="fuelPumpDataset"
        :dataset="graph_data"
        :yrange="[0, 300]"
        :xTitle="'Time (sec)'"
        :yTitle="'Pressure (PSI)'"
        :displayTimeSeconds="20.0"
        :displayTickInterval="2.0"
        :paddingFigs="3"
        class="columnLeft"
      />
      <RealtimeLineGraphChartjs
        :data-description="oxidizerPumpDataset"
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
        :data-description="inletDataset"
        :dataset="graph_data"
        :yrange="[0, 300]"
        :xTitle="'Time (sec)'"
        :yTitle="'Pressure (PSI)'"
        :displayTimeSeconds="20.0"
        :displayTickInterval="2.0"
        :paddingFigs="3"
        :justifyLegend="'left'"
        class="columnRight"
      />
    </div>
    <div class="row">
      <EmptyComponent class="columnLeft" />
      <RocketTerminal class="columnMiddle" id="terminal"/>
      <DatasetDisplay class="columnRight" :states="softwareDataset"/>
    </div>
  </div>
</template>

<script>
import RealtimeLineGraphChartjs from '../components/RealtimeLineGraphChartjs.vue';
import RocketTerminal from '../components/RocketTerminal.vue';
import DatasetDisplay from '../components/DatasetDisplay.vue';
import EmptyComponent from '@/components/EmptyComponent.vue';
import * as util from '../util/data.js';

export default {
  name: 'PumpTestPage',
  components: {
    RealtimeLineGraphChartjs,
    RocketTerminal,
    EmptyComponent,
    DatasetDisplay,
  },
  props: {
    refreshTimeMillis: {
      type: Number,
      default: 33
    },
  },
  computed: {
    fuelPumpDataset() {
      return [
        {
          name: 'F Outlet',
          color: 'orange',
          dataName: 'fuel_pump_outlet_pressure_psi',
          units: "PSI",
        },
        {
          name: 'F Inducer',
          color: 'red',
          dataName: 'fuel_pump_inducer_pressure_psi',
          units: "PSI",
        },
      ];
    },
    oxidizerPumpDataset() {
      return [
        {
          name: 'Ox Outlet',
          color: 'cyan',
          dataName: 'oxidizer_pump_outlet_pressure_psi',
          units: "PSI",
        },
        {
          name: 'Ox Inducer',
          color: 'blue',
          dataName: 'oxidizer_pump_inducer_pressure_psi',
          units: "PSI",
        },
      ];
    },
    inletDataset() {
      return [
        {
          name: 'F Inlet',
          color: 'orange',
          dataName: 'fuel_pump_inlet_pressure_psi',
          units: "PSI",
        },
        {
          name: 'Ox Inlet',
          color: 'cyan',
          dataName: 'oxidizer_pump_inlet_pressure_psi',
          units: "PSI",
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
