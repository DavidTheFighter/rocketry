<template>
  <div>
    <div class="row">
      <RealtimeLineGraphChartjs
        :data-description="altitudeDataset"
        :dataset="graph_data"
        :xTitle="'Time (sec)'"
        :yTitle="'Altitude AGL (meters)'"
        :displayTimeSeconds="20.0"
        :displayTickInterval="2.0"
        :paddingFigs="3"
        class="columnLeft"
      />
      <OrientationVisualization
        :orientation="rocketOrientation"
        class="columnMiddle"
      />
      <DatasetDisplay
        :states="numberDataset"
        class="columnRight"
      />
    </div>
    <div class="row">
      <RealtimeLineGraphChartjs
        :data-description="verticalVelocityDataset"
        :dataset="graph_data"
        :xTitle="'Time (sec)'"
        :yTitle="'Vertical Velocity (meters / sec)'"
        :displayTimeSeconds="20.0"
        :displayTickInterval="2.0"
        :paddingFigs="3"
        class="columnLeft"
      />
      <RocketTerminal class="columnMiddle"/>
      <EmptyComponent class="columnRight" />
    </div>
    <div class="row">
      <DatasetDisplay
        :states="debugInfo"
        :title="'Debug Info'"
        class="columnLeft"
      />
      <DatasetDisplay
        :states="this.dataset.problems"
        :title="'Problems'"
        :singleColumn="true"
        class="columnMiddle"
      />
      <EmptyComponent class="columnRight" />
    </div>
  </div>
</template>

<script>
import EmptyComponent from '../components/EmptyComponent.vue';
import RealtimeLineGraphChartjs from '../components/RealtimeLineGraphChartjs.vue';
import OrientationVisualization from '@/components/OrientationVisualization.vue';
import RocketTerminal from '../components/RocketTerminal.vue';
import DatasetDisplay from '../components/DatasetDisplay.vue';
import * as util from '../util/data.js';

export default {
  name: 'RocketPage',
  components: {
    EmptyComponent,
    RealtimeLineGraphChartjs,
    OrientationVisualization,
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
    altitudeDataset() {
      return [
        {
          name: 'Altitude',
          color: 'blue',
          dataName: 'altitude',
          units: "m",
        },
      ];
    },
    verticalVelocityDataset() {
      return [
        {
          name: 'y-Velocity',
          color: 'green',
          dataName: 'y_velocity',
          units: "m/s",
        },
      ];
    },
    numberDataset() {
      return [
        {
          name: "Vehicle State",
          value: this.dataset.vehicle_state,
        },
        {
          name: 'Altitude AGL',
          value: util.nelem(this.dataset.position, 1, -42.0).toFixed(1),
          units: "m",
          badValue: false,
        },
        {
          name: 'Telemetry Rate',
          value: this.dataset.telemetry_rate,
          units: "Hz",
        },
        {
          name: 'Telemetry Δt',
          value: Math.floor(util.nvalue(this.dataset.telemetry_delta_t)),
          units: "s",
          badValue: this.dataset.telemetry_delta_t >= 2.0,
        },
        {
          name: 'Battery Voltage',
          value: util.nvalue(this.dataset.battery_voltage).toFixed(1),
          units: "V",
        },
        {
          name: 'Speed',
          value: util.nmagnitude(this.dataset.velocity).toFixed(1),
          units: "m/s",
          badValue: false,
        },
        {
          name: 'Acceleration',
          value: util.nmagnitude(this.dataset.acceleration).toFixed(1),
          units: "m/s²",
          badValue: false,
        },
        {
          name: 'Angular Speed',
          value: (util.nmagnitude(this.dataset.angular_velocity) * util.RAD_TO_DEG).toFixed(1),
          units: "°/s",
          badValue: false,
        },
        {
          name: "Bytes logged",
          value: Math.floor(util.nvalue(this.dataset.data_logged_bytes) / 1024),
          units: "KiB",
          badValue: false,
        },
        {
          name: "Apogee",
          value: util.nvalue(this.dataset.apogee).toFixed(1),
          units: "m",
          badValue: false,
        },
        {
          name: "Bitrate",
          value: (util.nvalue(this.dataset.fcu_bitrate) / 1024.0).toFixed(1),
          units: "kbps",
          badValue: false,
        }
      ];
    },
    debugInfo() {
      return [
        {
          name: "Raw Accel",
          value: util.nvecstr(this.dataset.debug_data?.raw_accelerometer, 0),
          badValue: false,
        },
        {
          name: "Raw Gyro",
          value: util.nvecstr(this.dataset.debug_data?.raw_gyroscope, 0),
          badValue: false,
        },
        {
          name: "Raw Magno",
          value: util.nvecstr(this.dataset.debug_data?.raw_magnetometer, 0),
          badValue: false,
        },
        {
          name: "Raw Baro",
          value: this.dataset.debug_data?.raw_barometer,
          badValue: false,
        },
        {
          name: "Baro Alt",
          value: util.nvalue(this.dataset.debug_data?.barometric_altitude).toFixed(2),
          units: "m",
          badValue: false,
        },
        {
          name: "Accel Calib",
          value: util.nvecstr(this.dataset.debug_data?.accelerometer_calibration, 0),
          badValue: false,
        },
        {
          name: "Baro Calib",
          value: this.dataset.debug_data?.barometer_calibration,
          badValue: false,
        },
      ];
    },
    rocketOrientation() {
      return this.dataset.orientation;
    },
  },
  watch: {
    timer: {
      async handler() {
        this.generateData();

        // setTimeout(() => {
        //   this.timer += 1;
        // }, this.refreshTimeMillis);
      },
      immediate: true,
    }
  },
  methods: {
    async generateData() {
      let debug_data = undefined;

      try {
        debug_data = await this.fetcher.fetch('http://localhost:8000/fcu-telemetry/debug-data');
      } catch (error) {
        console.log(error);
      }

      try {
        let dataset = await this.fetcher.fetch('http://localhost:8000/fcu-telemetry');
        dataset.debug_data = debug_data;

        this.dataset = dataset;
      } catch (error) {
        console.log(error);
      }

      try {
        this.graph_data = await this.fetcher.fetch('http://localhost:8000/fcu-telemetry/graph');
      } catch (error) {
        console.log(error);
      }

      this.timer += 1;
    },
  },
  created() {
    this.fetcher = new util.DataFetcher(this.refreshTimeMillis / 2);
  },
  data() {
    return {
      timer: 0,
      dataset: {},
      graph_data: {},
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

.row {
  display: flex;
  margin-bottom: 5px;
  height: 33vh;
}

.columnLeft {
  flex: 33%;
  padding-right: 10px;
}

.columnMiddle {
  flex: 33%;
  padding-left: 10px;
  padding-right: 10px;
}

.columnRight {
  flex: 33%;
  max-height: 100%;
  padding-left: 10px;
}

.columnRightTwoThirds {
  flex: 67%;
  max-width: 66%;
}

</style>
