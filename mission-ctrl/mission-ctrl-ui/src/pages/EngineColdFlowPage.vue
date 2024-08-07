<template>
  <div>
    <div class="row">
      <RealtimeLineGraph
        :data-description="tankDataset"
        :dataset="dataset"
        :yrange="[0, 100]"
        :xTitle="'Time (sec)'"
        :displayTimeSeconds="30.0"
        :displayTickInterval="5.0"
        class="columnFourths"
      />
      <RealtimeLineGraph
        :data-description="pumpDataset"
        :dataset="dataset"
        :yrange="[0, 800]"
        :xTitle="'Time (sec)'"
        :displayTimeSeconds="30.0"
        :displayTickInterval="5.0"
        class="columnFourths"
      />
      <RealtimeLineGraph
        :data-description="igniterDataset"
        :dataset="dataset"
        :yrange="[0, 800]"
        :xTitle="'Time (sec)'"
        :yTitle="'Pressure (PSI)'"
        :displayTimeSeconds="30.0"
        :displayTickInterval="5.0"
        class="columnFourths"
      />
      <RealtimeLineGraph
        :data-description="engineDataset"
        :dataset="dataset"
        :yrange="[0, 800]"
        :xTitle="'Time (sec)'"
        :displayTimeSeconds="30.0"
        :displayTickInterval="5.0"
        class="columnFourths"
      />
    </div>
    <div class="row">
      <DatasetDisplay class="columnThirds" :states="valveDataset"/>
      <TtydTerminal class="columnThirds"/>
      <DatasetDisplay class="columnThirds" :states="softwareDataset"/>
    </div>
    <div class="row">
      <EmptyComponent class="columnThirds" />
      <AlertDisplay
        :dataset="dataset"
        :title="'Alerts'"
        class="columnThirds"
      />
      <EmptyComponent class="columnThirds" />
    </div>
  </div>
</template>

<script>
import RealtimeLineGraph from '../components/RealtimeLineGraph.vue';
import TtydTerminal from '../components/TtydTerminal.vue';
import DatasetDisplay from '../components/DatasetDisplay.vue';
import AlertDisplay from '../components/AlertDisplay.vue';
import EmptyComponent from '../components/EmptyComponent.vue';
import * as util from '../util/data.js';

export default {
  name: 'EngineColdFlow',
  components: {
    RealtimeLineGraph,
    TtydTerminal,
    DatasetDisplay,
    AlertDisplay,
    EmptyComponent,
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
          name: 'IG Chamb',
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
          name: 'Fl Tank',
          color: 'orange',
          fieldName: "tank_telemetry.fuel_tank_pressure_pa",
          units: "PSIA",
          scale: 0.00014503773773020923,
        },
        {
          name: 'Ox Tank',
          color: 'cyan',
          fieldName: "tank_telemetry.oxidizer_tank_pressure_pa",
          units: "PSIA",
          scale: 0.00014503773773020923,
        },
      ];
    },
    pumpDataset() {
      return [
        {
          name: 'Fl Pump',
          color: 'orange',
          fieldName: 'telemetry.fuel_pump_outlet_pressure_pa',
          units: "PSIA",
          scale: 0.00014503773773020923,
        },
        {
          name: 'Ox Pump',
          color: 'cyan',
          fieldName: 'telemetry.oxidizer_pump_outlet_pressure_pa',
          units: "PSIA",
          scale: 0.00014503773773020923,
        },
      ];
    },
    engineDataset() {
      return [
      {
          name: 'Ox Inj',
          color: 'cyan',
          fieldName: 'telemetry.engine_oxidizer_injector_pressure_pa',
          units: "PSIA",
          scale: 0.00014503773773020923,
        },
        {
          name: 'Fl Inj',
          color: 'orange',
          fieldName: 'telemetry.engine_fuel_injector_pressure_pa',
          units: "PSIA",
          scale: 0.00014503773773020923,
        },
        {
          name: 'Eng Chamb',
          color: 'red',
          fieldName: 'telemetry.engine_chamber_pressure_pa',
          units: "PSIA",
          scale: 0.00014503773773020923,
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
          lastValue: this.dataset?.telemetry?.engine_state,
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
          lastValue: this.dataset?.tank_telemetry?.oxidizer_tank_state,
        },
        {
          name: 'Fuel Pump State',
          lastValue: this.dataset?.telemetry?.fuel_pump_state,
        },
        {
          name: 'Ox Pump State',
          lastValue: this.dataset?.telemetry?.oxidizer_pump_state,
        },
        {
          name: 'Telemetry Rate',
          value: this.dataset?.display_fields?.telemetry_rate_hz,
          units: "Hz",
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
