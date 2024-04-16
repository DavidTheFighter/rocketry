<template>
  <div>
    <div class="row">
      <RealtimeLineGraphChartjs
        :data-description="igniterDataset"
        :dataset="graph_data"
        :yrange="[0, 800]"
        :xTitle="'Time (sec)'"
        :yTitle="'Pressure (PSI)'"
        :displayTimeSeconds="20.0"
        :displayTickInterval="2.0"
        class="columnFourths"
      />
      <RealtimeLineGraphChartjs
        :data-description="tankDataset"
        :dataset="graph_data"
        :yrange="[0, 100]"
        :xTitle="'Time (sec)'"
        :displayTimeSeconds="20.0"
        :displayTickInterval="2.0"
        class="columnFourths"
      />
      <RealtimeLineGraphChartjs
        :data-description="pumpDataset"
        :dataset="graph_data"
        :yrange="[0, 800]"
        :xTitle="'Time (sec)'"
        :displayTimeSeconds="20.0"
        :displayTickInterval="2.0"
        class="columnFourths"
      />
      <RealtimeLineGraphChartjs
        :data-description="engineDataset"
        :dataset="graph_data"
        :yrange="[0, 800]"
        :xTitle="'Time (sec)'"
        :displayTimeSeconds="20.0"
        :displayTickInterval="2.0"
        class="columnFourths"
      />
    </div>
    <div class="row">
      <DatasetDisplay class="columnThirds" :states="valveDataset"/>
      <RocketTerminal class="columnThirds" id="terminal"/>
      <DatasetDisplay class="columnThirds" :states="softwareDataset"/>
    </div>
  </div>
</template>

<script>
import RealtimeLineGraphChartjs from '../components/RealtimeLineGraphChartjs.vue';
import RocketTerminal from '../components/RocketTerminal.vue';
import DatasetDisplay from '../components/DatasetDisplay.vue';
import * as util from '../util/data.js';

export default {
  name: 'EngineColdFlow',
  components: {
    RealtimeLineGraphChartjs,
    RocketTerminal,
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
          units: "PSIA",
        },
        {
          name: 'IG Fuel',
          color: 'orange',
          dataName: 'igniter_fuel_pressure_psi',
          units: "PSIA",
        },
        {
          name: 'IG Chamb',
          color: 'red',
          dataName: 'igniter_chamber_pressure_psi',
          units: "PSIA",
        },
      ];
    },
    tankDataset() {
      return [
        {
          name: 'Fl Tank',
          color: 'orange',
          dataName: "fuel_tank_pressure_psi",
          units: "PSIA",
        },
        {
          name: 'Ox Tank',
          color: 'cyan',
          dataName: "oxidizer_tank_pressure_psi",
          units: "PSIA",
        },
      ];
    },
    pumpDataset() {
      return [
        {
          name: 'Fl Pump',
          color: 'orange',
          dataName: 'fuel_pump_outlet_pressure_psi',
          units: "PSIA",
        },
        {
          name: 'Ox Pump',
          color: 'cyan',
          dataName: 'oxidizer_pump_outlet_pressure_psi',
          units: "PSIA",
        },
      ];
    },
    engineDataset() {
      return [
        {
          name: 'Fl Inj',
          color: 'orange',
          dataName: 'engine_fuel_injector_pressure_psi',
          units: "PSIA",
        },
        {
          name: 'Ox Inj',
          color: 'cyan',
          dataName: 'engine_oxidizer_injector_pressure_psi',
          units: "PSIA",
        },
        {
          name: 'Eng Chamb',
          color: 'red',
          dataName: 'engine_chamber_pressure_psi',
          units: "PSIA",
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
          name: 'Engine State',
          value: this.dataset.engine_state,
        },
        {
          name: 'Igniter State',
          value: this.dataset.igniter_state,
        },
        {
          name: 'Fuel Tank State',
          value: this.dataset.fuel_tank_state,
        },
        {
          name: 'Ox Tank State',
          value: this.dataset.oxidizer_tank_state,
        },
        {
          name: 'Fuel Pump State',
          value: this.dataset.fuel_pump_state,
        },
        {
          name: 'Ox Pump State',
          value: this.dataset.oxidizer_pump_state,
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
        debug_data = await this.fetcher.fetch('http://127.0.0.1:8000/ecu-telemetry/0/debug-data');
      } catch (error) {
        console.log(error);
      }

      try {
        let dataset = await this.fetcher.fetch('http://127.0.0.1:8000/ecu-telemetry/0');
        dataset.debug_data = debug_data;

        this.dataset = dataset;
      } catch (error) {
        console.log(error);
      }

      try {
        this.graph_data = await this.fetcher.fetch('http://127.0.0.1:8000/ecu-telemetry/0/graph');
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
  height: 20rem;
}

.row {
  display: flex;
  margin-bottom: 20px;
}

.columnThirds {
  flex: 33%;
  padding-left: 15px;
  padding-right: 15px;
}

.columnFourths {
  flex: 25%;
}

.columnRightTwoThirds {
  flex: 67%;
  max-width: 66%;
}

</style>
