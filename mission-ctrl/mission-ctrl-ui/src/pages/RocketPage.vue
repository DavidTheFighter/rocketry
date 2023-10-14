<template>
  <div>
    <div class="row">
      <RealtimeLineGraphChartjs
        :datasets="altitudeDataset"
        :xTitle="'Time (sec)'"
        :yTitle="'Altitude AGL (meters)'"
        :numXTicks="15"
        :scaleXTicks="2"
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
        :datasets="verticalVelocityDataset"
        :xTitle="'Time (sec)'"
        :yTitle="'Vertical Velocity (meters / sec)'"
        :numXTicks="15"
        :scaleXTicks="2"
        :paddingFigs="3"
        class="columnLeft"
      />
      <RocketTerminal class="columnMiddle"/>
      <EmptyComponent class="columnRight" />
    </div>
    <div class="row">
      <EmptyComponent class="columnLeft" />
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
          data: this.dataset?.graph_data?.altitude,
          units: "m",
        },
      ];
    },
    verticalVelocityDataset() {
      return [
        {
          name: 'y-Velocity',
          color: 'green',
          data: this.dataset?.graph_data?.y_velocity,
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
          name: 'Battery Voltage',
          value: this.nzero(this.dataset.battery_voltage).toFixed(1),
          units: "V",
        },
        {
          name: 'Telemetry Rate',
          value: this.dataset.telemetry_rate,
          units: "Hz",
        },
        {
          name: 'Telemetry Δt',
          value: Math.floor(this.nzero(this.dataset.telemetry_delta_t)),
          units: "s",
          badValue: this.dataset.telemetry_delta_t >= 2.0,
        },
        {
          name: 'Altitude AGL',
          value: this.nzero(this.lastElementOrNull(this.dataset.altitude)).toFixed(1),
          units: "m",
          badValue: false,
        },
        {
          name: 'Speed',
          value: this.nzero(this.dataset.speed).toFixed(1),
          units: "m/s",
          badValue: false,
        },
        {
          name: 'Acceleration',
          value: this.accelerationStr(),
          // units: "m/s^2",
        },
        {
          name: 'Angular Velocity',
          value: this.angularVelocityStr(),
          // units: "m/s^2",
        },
        {
          name: 'Magnetic Field',
          value: this.magneticFieldStr(),
          // units: "m/s^2",
        },
        {
          name: '|Magnetic Field|',
          value: this.magneticFieldLStr(),
          // units: "m/s^2",
        },
        {
          name: "Bytes logged",
          value: Math.floor(this.nzero(this.dataset.bytes_logged) / 1024),
          units: "KiB",
          badValue: false,
        },
        {
          name: "Barometric",
          value: this.dataset.detailed_state?.barometric_pressure,
          units: "Pa",
          badValue: false,
        }
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

        setTimeout(() => {
          this.timer += 1;
        }, this.refreshTimeMillis);
      },
      immediate: true,
    }
  },
  methods: {
    async generateData() {
      try {
        const response = await fetch('http://localhost:8000/fcu-telemetry');
        const data = await response.json();

        this.dataset = data;
      } catch (error) {
        console.log(error);

        this.dataset = [];
      }
    },
    lastElementOrNull(array) {
      if (array) {
        return array[array.length - 1];
      } else {
        return null;
      }
    },
    nzero(value) {
      if (value != null && value != undefined) {
        return value;
      } else {
        return 0;
      }
    },
    toFixedOrZero(value, precision) {
      if (value != null && value != undefined) {
        return value.toFixed(precision);
      } else {
        return "0";
      }
    },
    accelerationStr() {
      if (this.dataset.acceleration) {
        const x = this.toFixedOrZero(this.dataset.acceleration[0], 2);
        const y = this.toFixedOrZero(this.dataset.acceleration[1], 2);
        const z = this.toFixedOrZero(this.dataset.acceleration[2], 2);

        return `(${x}, ${y}, ${z})`;
      }

      return "(0.0, 0.0, 0.0)";
    },
    angularVelocityStr() {
      if (this.dataset.angular_velocity) {
        const x = this.toFixedOrZero(this.dataset.angular_velocity[0], 3);
        const y = this.toFixedOrZero(this.dataset.angular_velocity[1], 3);
        const z = this.toFixedOrZero(this.dataset.angular_velocity[2], 3);

        return `(${x}, ${y}, ${z})`;
      }

      return "(0.0, 0.0, 0.0)";
    },
    magneticFieldStr() {
      // if (this.dataset.magnetic_field) {
      //   const x = this.dataset.magnetic_field[0].toFixed(3);
      //   const y = this.dataset.magnetic_field[1].toFixed(3);
      //   const z = this.dataset.magnetic_field[2].toFixed(3);

      //   return `(${x}, ${y}, ${z})`;
      // }

      return "(?, ?, ?)";
    },
    magneticFieldLStr() {
      if (this.dataset.magnetic_field) {
        const x = this.dataset.magnetic_field[0];
        const y = this.dataset.magnetic_field[1];
        const z = this.dataset.magnetic_field[2];
        const length = Math.sqrt(x * x + y * y + z * z);

        return length.toFixed(3);
      }

      return "?";
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
