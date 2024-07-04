<template>
  <div>
    <div class="row">
      <RealtimeLineGraph
        :data-description="igniterDataset"
        :dataset="dataset"
        :yrange="[0, 300]"
        :xTitle="'Time (sec)'"
        :displayTimeSeconds="30.0"
        :displayTickInterval="5.0"
        class="columnLeft"
      />
      <RealtimeLineGraph
        :data-description="tankDataset"
        :dataset="dataset"
        :yrange="[0, 300]"
        :xTitle="'Time (sec)'"
        :displayTimeSeconds="30.0"
        :displayTickInterval="5.0"
        :justifyLegend="'left'"
        class="columnMiddle"
      />
      <RealtimeLineGraph
        :data-description="tankDataset"
        :dataset="dataset"
        :xTitle="'Time (sec)'"
        :displayTimeSeconds="30.0"
        :displayTickInterval="5.0"
        :justifyLegend="'left'"
        class="columnLeft"
      />
    </div>
    <div class="row">
      <DatasetDisplay class="columnLeft" :states="valveDataset"/>
      <RocketTerminal class="columnMiddle" id="terminal"/>
      <DatasetDisplay class="columnRight" :states="softwareDataset"/>
    </div>
  </div>
</template>

<script>
import RealtimeLineGraph from '../components/RealtimeLineGraph.vue';
import RocketTerminal from '../components/RocketTerminal.vue';
import DatasetDisplay from '../components/DatasetDisplay.vue';
import * as util from '../util/data.js';

export default {
  name: 'IgniterPage',
  components: {
    RealtimeLineGraph,
    RocketTerminal,
    DatasetDisplay,
  },
  computed: {
    igniterDataset() {
      return [
        {
          name: 'IG GOx',
          color: 'cyan',
          fieldName: 'sensors.IgniterOxidizerInjectorPressure',
          units: "PSIA",
          scale: 0.00014503773773020923,
        },
        {
          name: 'IG Fuel',
          color: 'orange',
          fieldName: 'sensors.IgniterFuelInjectorPressure',
          units: "PSIA",
          scale: 0.00014503773773020923,
        },
        {
          name: 'IG Chamber',
          color: 'red',
          fieldName: 'telemetry.igniter_chamber_pressure_pa',
          units: "PSIA",
          scale: 0.00014503773773020923,
        },
      ];
    },
    tankDataset() {
      return [
        {
          name: 'Fuel Tank',
          color: 'orange',
          fieldName: "tank_telemetry.fuel_tank_pressure_pa",
          units: "PSIA",
          scale: 0.00014503773773020923,
        },
        {
          name: 'Oxidizer Tank',
          color: 'cyan',
          fieldName: "tank_telemetry.oxidizer_tank_pressure_pa",
          units: "PSIA",
          scale: 0.00014503773773020923,
        },
      ];
    },
    tempDataset() {
      return [
        {
          name: 'ECU Board',
          color: '#7FFFD4',
          value: this.dataset?.ecu_board_temp,
          units: "°C",
        },
        {
          name: 'IG Throat',
          color: '#A9A9A9',
          value: this.dataset?.igniter_throat_temp,
          units: "°C",
        },
      ];
    },
    valveDataset() {
      return [
        {
          name: 'IG Fuel Valve',
          value: this.dataset?.igniter_fuel_valve,
        },
        {
          name: 'IG GOx Valve',
          value: this.dataset?.igniter_gox_valve,
        },
        {
          name: 'Fuel Press Valve',
          value: this.dataset?.fuel_press_valve,
        },
        {
          name: 'Fuel Vent Valve',
          value: this.dataset?.fuel_vent_valve,
        },
        {
          name: 'IG Spark Plug',
          value: this.dataset?.sparking,
        },
      ];
    },
    softwareDataset() {
      return [
      {
          name: 'Telemetry Rate',
          value: this.dataset?.display_fields?.telemetry_rate_hz,
          units: "Hz",
        },
        {
          name: 'Igniter State',
          lastValue: this.dataset?.telemetry?.igniter_state,
        },
        {
          name: 'Fuel Tank State',
          lastValue: this.dataset?.tank_telemetry?.fuel_tank_state,
        },
        {
          name: 'Ox Tank State',
          lastValue: this.dataset?.tank_telemetry?.fuel_tank_state,
        },
      ];
    },
  },
  mounted() {
    this.data_handler = new util.DataHandler(`http://${window.location.hostname}:8000/ecu-telemetry-stream/0`, this.dataset);
  },
  data() {
    return {
      dataset: {},
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
