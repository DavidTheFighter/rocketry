<template>
  <div id="page">
    <div class="row">
      <RealtimeLineGraphChartjs
        :datasets="positionDataset"
        :dataset="dataset"
        :xTitle="'Position (m)'"
        :numXTicks="10"
        :scaleXTicks="2"
        :paddingFigs="0"
        :label-legend="false"
        class="columnThreeSixteenth"
      />
      <RealtimeLineGraphChartjs
        :datasets="velocityDataset"
        :dataset="dataset"
        :xTitle="'Velocity (m/s)'"
        :numXTicks="10"
        :scaleXTicks="2"
        :paddingFigs="0"
        :label-legend="false"
        class="columnThreeSixteenth"
      />
      <RealtimeLineGraphChartjs
        :datasets="accelerationDataset"
        :dataset="dataset"
        :xTitle="'Acceleration (m/s^2)'"
        :numXTicks="10"
        :scaleXTicks="2"
        :paddingFigs="0"
        :label-legend="false"
        class="columnThreeSixteenth"
      />
      <RealtimeLineGraphChartjs
        :datasets="angularVelocityDataset"
        :dataset="dataset"
        :xTitle="'Angular Velocity (°/s)'"
        :numXTicks="10"
        :scaleXTicks="2"
        :paddingFigs="0"
        :label-legend="false"
        class="columnThreeSixteenth"
      />
      <OrientationVisualization
        :orientation="rocketOrientation"
        class="columnFourth"
      />
    </div>
    <div class="row">
      <RealtimeLineGraphChartjs
        :datasets="dpositionDataset"
        :dataset="dataset"
        :xTitle="'Δ Position XZ/Y (m)'"
        :numXTicks="10"
        :scaleXTicks="2"
        :paddingFigs="0"
        :label-legend="false"
        class="columnThreeSixteenth"
      />
      <RealtimeLineGraphChartjs
        :datasets="dvelocityDataset"
        :dataset="dataset"
        :xTitle="'Δ Velocity XZ/Y (m/s)'"
        :numXTicks="10"
        :scaleXTicks="2"
        :paddingFigs="0"
        :label-legend="false"
        class="columnThreeSixteenth"
      />
      <RealtimeLineGraphChartjs
        :datasets="dOrientationDataset"
        :dataset="dataset"
        :xTitle="'Δ Orientation (°)'"
        :numXTicks="10"
        :scaleXTicks="2"
        :paddingFigs="0"
        :label-legend="false"
        class="columnThreeSixteenth"
      />
      <RealtimeLineGraphChartjs
        :datasets="dangularVelocityDataset"
        :dataset="dataset"
        :xTitle="'Δ Angular Velocity (°/s)'"
        :numXTicks="10"
        :scaleXTicks="2"
        :paddingFigs="0"
        :label-legend="false"
        class="columnThreeSixteenth"
      />
      <DatasetDisplay
        :states="numberDataset"
        class="columnFourth"
      />
    </div>
    <div class="row">
      <RealtimeLineGraphChartjs
        :datasets="fcuPositionDataset"
        :dataset="dataset"
        :xTitle="'FCU Position (m)'"
        :numXTicks="10"
        :scaleXTicks="2"
        :paddingFigs="0"
        :label-legend="false"
        class="columnFourth"
      />
      <RealtimeLineGraphChartjs
        :datasets="fcuVelocityDataset"
        :dataset="dataset"
        :xTitle="'FCU Velocity (m/s)'"
        :numXTicks="10"
        :scaleXTicks="2"
        :paddingFigs="0"
        :label-legend="false"
        class="columnFourth"
      />
      <RealtimeLineGraphChartjs
        :datasets="fcuAccelerationDataset"
        :dataset="dataset"
        :xTitle="'FCU Acceleration (m/s^2)'"
        :numXTicks="10"
        :scaleXTicks="2"
        :paddingFigs="0"
        :label-legend="false"
        class="columnFourth"
      />
      <EmptyComponent class="columnFourth" />
    </div>
  </div>
</template>

<script>
import EmptyComponent from '../components/EmptyComponent.vue';
// import RealtimeLineGraph from '../components/RealtimeLineGraph.vue';
import RealtimeLineGraphChartjs from '../components/RealtimeLineGraphChartjs.vue';
import OrientationVisualization from '@/components/OrientationVisualization.vue';
import DatasetDisplay from '../components/DatasetDisplay.vue';

export default {
  name: 'RocketSimPage',
  components: {
    EmptyComponent,
    // RealtimeLineGraph,
    RealtimeLineGraphChartjs,
    OrientationVisualization,
    DatasetDisplay,
  },
  props: {
    refreshTimeMillis: {
      type: Number,
      default: 33,
    },
  },
  computed: {
    positionDataset() {
      return [
        {
          name: 'X',
          color: '#D00',
          dataName: 'position',
          dataIndex: 0,
          units: "m",
        },
        {
          name: 'Y',
          color: '#0BF',
          dataName: 'position',
          dataIndex: 1,
          units: "m",
        },
        {
          name: 'Z',
          color: '#0B0',
          dataName: 'position',
          dataIndex: 2,
          units: "m",
        },
      ];
    },
    velocityDataset() {
      return [
      {
          name: 'X',
          color: '#D00',
          dataName: 'velocity',
          dataIndex: 0,
          units: "m",
        },
        {
          name: 'Y',
          color: '#0BF',
          dataName: 'velocity',
          dataIndex: 1,
          units: "m",
        },
        {
          name: 'Z',
          color: '#0B0',
          dataName: 'velocity',
          dataIndex: 2,
          units: "m",
        },
      ];
    },
    accelerationDataset() {
      return [
        {
          name: 'X',
          color: '#D00',
          dataName: 'acceleration',
          dataIndex: 0,
          units: "m",
        },
        {
          name: 'Y',
          color: '#0BF',
          dataName: 'acceleration',
          dataIndex: 1,
          units: "m",
        },
        {
          name: 'Z',
          color: '#0B0',
          dataName: 'acceleration',
          dataIndex: 2,
          units: "m",
        },
      ];
    },
    angularVelocityDataset() {
      return [
        {
          name: 'X',
          color: '#D00',
          dataName: 'angular_velocity',
          dataIndex: 0,
          units: "°/s",
          scale: 57.29578,
        },
        {
          name: 'Y',
          color: '#0BF',
          dataName: 'angular_velocity',
          dataIndex: 1,
          units: "°/s",
          scale: 57.29578,
        },
        {
          name: 'Z',
          color: '#0B0',
          dataName: 'angular_velocity',
          dataIndex: 2,
          units: "°/s",
          scale: 57.29578,
        },
      ];
    },
    dpositionDataset() {
      return [
        {
          name: 'XZ',
          color: '#D00',
          dataName: 'dposition',
          dataIndex: 0,
          units: "m",
        },
        {
          name: 'Y',
          color: '#0BF',
          dataName: 'dposition',
          dataIndex: 1,
          units: "m",
        },
        {
          name: 'Z',
          color: '#0B0',
          dataName: 'dposition',
          dataIndex: 2,
          units: "m",
        }
      ];
    },
    dvelocityDataset() {
      return [
      {
          name: 'XZ',
          color: '#D00',
          dataName: 'dvelocity',
          dataIndex: 0,
          units: "m",
        },
        {
          name: 'Y',
          color: '#0BF',
          dataName: 'dvelocity',
          dataIndex: 1,
          units: "m",
        },
        {
          name: 'Z',
          color: '#0B0',
          dataName: 'dvelocity',
          dataIndex: 3,
          units: "m",
        }
      ];
    },
    dOrientationDataset() {
      return [
        {
          name: 'X',
          color: '#0BF',
          dataName: 'dorientation',
          dataIndex: 0,
          units: "°",
        }
      ];
    },
    dangularVelocityDataset() {
      return [
        {
          name: 'X',
          color: '#D00',
          dataName: 'dangular_velocity',
          dataIndex: 0,
          units: "°/s",
        },
        {
          name: 'Y',
          color: '#0BF',
          dataName: 'dangular_velocity',
          dataIndex: 1,
          units: "°/s",
        },
        {
          name: 'Z',
          color: '#0B0',
          dataName: 'dangular_velocity',
          dataIndex: 2,
          units: "°/s",
        }
      ];
    },
    fcuPositionDataset() {
      return [
        {
          name: 'X',
          color: '#D00',
          dataName: 'fcu_position',
          dataIndex: 0,
          units: "m",
        },
        {
          name: 'Y',
          color: '#0BF',
          dataName: 'fcu_position',
          dataIndex: 1,
          units: "m",
        },
        {
          name: 'Z',
          color: '#0B0',
          dataName: 'fcu_position',
          dataIndex: 2,
          units: "m",
        },
      ];
    },
    fcuVelocityDataset() {
      return [
        {
          name: 'X',
          color: '#D00',
          dataName: 'fcu_velocity',
          dataIndex: 0,
          units: "m",
        },
        {
          name: 'Y',
          color: '#0BF',
          dataName: 'fcu_velocity',
          dataIndex: 1,
          units: "m",
        },
        {
          name: 'Z',
          color: '#0B0',
          dataName: 'fcu_velocity',
          dataIndex: 2,
          units: "m",
        },
      ];
    },
    fcuAccelerationDataset() {
      return [
        {
          name: 'X',
          color: '#D00',
          dataName: 'fcu_acceleration',
          dataIndex: 0,
          units: "m",
        },
        {
          name: 'Y',
          color: '#0BF',
          dataName: 'fcu_acceleration',
          dataIndex: 1,
          units: "m",
        },
        {
          name: 'Z',
          color: '#0B0',
          dataName: 'fcu_acceleration',
          dataIndex: 2,
          units: "m",
        },
      ];
    },
    emptyDataset() {
      return [
        {
          name: 'f',
          color: 'red',
          units: "m",
        }
      ];
    },
    numberDataset() {
      return [
        {
          name: "Vehicle State",
          value: this.dataset.telemetry?.vehicle_state,
        },
        // {
        //   name: 'Battery Voltage',
        //   value: this.nzero(this.dataset.telemetry?.battery_voltage).toFixed(1),
        //   units: "V",
        // },
        {
          name: 'Time',
          value: this.nzero(this.dataset.sim_data?.time).toFixed(1),
          units: 's',
          badValue: false,
        },
        {
          name: 'Sim Speed',
          value: this.nzero(this.dataset.sim_data?.speed).toFixed(1),
          units: "m/s",
          badValue: false,
        },
        // {
        //   name: 'FCU Speed',
        //   value: this.nzero(this.dataset.sim_data?.measured_speed).toFixed(1),
        //   units: "m/s",
        //   badValue: false,
        // },
        {
          name: 'Apogee',
          value: this.nzero(this.dataset.telemetry?.apogee).toFixed(1),
          units: 'm',
          badValue: false,
        },
        {
          name: 'XZ-σ',
          value: this.positionXZErrorStr(),
          badValue: false,
        },
        {
          name: 'Y-σ',
          value: this.indexFixed(this.dataset.detailed_state?.position_error, 1, 2),
          badValue: false,
        },
        {
          name: 'XZv-σ',
          value: this.velocityXZErrorStr(),
          badValue: false,
        },
        {
          name: 'Yv-σ',
          value: this.indexFixed(this.dataset.detailed_state?.velocity_error, 1, 2),
          badValue: false,
        },
        {
          name: 'XYZa-σ',
          value: this.accelerationXYZErrorStr(),
          badValue: false,
        },
      ];
    },
    rocketOrientation() {
      return this.dataset?.telemetry?.orientation;
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
        this.dataset = await this.timeoutFetch('http://localhost:5000/simdata', this.refreshTimeMillis - 1);
      } catch (error) {
        this.dataset = [];
      }
    },
    async timeoutFetch(url, timeout) {
      const controller = new AbortController()
      const timeoutId = setTimeout(() => {
        controller.abort()
      }, timeout)

      const response = await fetch(url, {
        signal: controller.signal,
      })

      clearTimeout(timeoutId)
      const data = await response.json()
      return data
    },
    lastElementOrNull(array) {
      if (array) {
        return array[array.length - 1];
      } else {
        return null;
      }
    },
    ithElementOrNull(array, i) {
      if (array) {
          return array[i];
      } else {
          return null;
      }
    },
    ithElementOrEmpty(array, i) {
      if (array) {
          return array[i];
      } else {
          return [];
      }
    },
    nzero(value) {
      if (value != null && value != undefined) {
        return value;
      } else {
        return 0;
      }
    },
    nzeroIndex(array, index) {
      if (array == null || array == undefined || array.length == null || array.length == undefined) {
        return 0;
      }

      if (index >= array.length || array[index] == null || array[index] == undefined) {
        return 0;
      }

      return array[index];
    },
    toFixedOrZero(value, precision) {
      if (value != null && value != undefined) {
        return value.toFixed(precision);
      } else {
        return "0";
      }
    },
    indexFixed(value, index, precision) {
      if (value == null || value == undefined || value.length == null || value.length == undefined) {
        return "0";
      }

      if (index >= value.length) {
        return "0";
      }

      return value[index].toFixed(precision);
    },
    positionXZErrorStr() {
      const x = this.indexFixed(this.dataset.detailed_state?.position_error, 0, 2);
      const z = this.indexFixed(this.dataset.detailed_state?.position_error, 2, 2);
      return `${x}, ${z}`;
    },
    velocityXZErrorStr() {
      const x = this.indexFixed(this.dataset.detailed_state?.velocity_error, 0, 2);
      const z = this.indexFixed(this.dataset.detailed_state?.velocity_error, 2, 2);
      return `${x}, ${z}`;
    },
    accelerationXYZErrorStr() {
      const x = this.nzeroIndex(this.dataset.detailed_state?.acceleration_error, 0);
      const y = this.nzeroIndex(this.dataset.detailed_state?.acceleration_error, 1);
      const z = this.nzeroIndex(this.dataset.detailed_state?.acceleration_error, 2);
      const error = Math.sqrt(x*x + y*y + z*z).toFixed(2);
      return `${error}`;
    },
    accelerometerBiasStr() {
      const x = this.nzeroIndex(this.dataset.detailed_state?.accelerometer_bias, 0).toFixed(2);
      const y = this.nzeroIndex(this.dataset.detailed_state?.accelerometer_bias, 1).toFixed(2);
      const z = this.nzeroIndex(this.dataset.detailed_state?.accelerometer_bias, 2).toFixed(2);
      return `${x}, ${y}, ${z}`;
    },
  },
  data() {
    return {
      timer: 0,
      dataset: {},
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
  height: 25vh;
}

.columnFourth {
  flex: 25%;
  padding-left: 3px;
  padding-right: 3px;
}

.columnThreeSixteenth {
  flex: 18.75%;
  padding-left: 3px;
  padding-right: 3px;
}

</style>
