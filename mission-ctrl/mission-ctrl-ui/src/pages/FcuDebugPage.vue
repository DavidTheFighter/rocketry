<template>
  <div>
    <div class="row">
      <DatasetDisplay
        :states="debugInfo"
        class="columnLeft"
      />
      <EmptyComponent class="columnRight" />
    </div>
  </div>
</template>

<script>
import EmptyComponent from '../components/EmptyComponent.vue';
import DatasetDisplay from '../components/DatasetDisplay.vue';
import * as util from '../util/data.js';

export default {
  name: 'FcuDebugPage',
  components: {
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
    debugInfo() {
      let debugInfo = [];

      if (this.dataset.debug_data) {
        debugInfo = Object.entries(this.dataset.debug_data).map(([key, value]) => {
          if (typeof value === 'number') {
            if (Number.isInteger(value)) {
              value = value.toString();
            } else {
              value = value.toFixed(2);
            }
          } else if (Array.isArray(value)) {
            let str_value = '[';

            value.forEach((val) => {
              if (typeof val === 'number') {
                if (Number.isInteger(val)) {
                  str_value += val + ', ';
                } else {
                  str_value += val.toFixed(2) + ', ';
                }
              } else {
                str_value += val + ', ';
              }
            });

            str_value = str_value.slice(0, -2) + ']';

            value = str_value;

            // value = value.map((val) => {
            //   if (typeof val === 'number') {
            //     return val.toFixed(2);
            //   } else {
            //     return val;
            //   }
            // });
          }

          return {
            name: key,
            value: value,
            badValue: false,
          };
        });
      }

      return debugInfo;
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
      let debug_data = undefined;

      try {
        debug_data = await util.timeoutFetch('http://localhost:8000/fcu-telemetry/debug-data', this.refreshTimeMillis - 1);
      } catch (error) {
        console.log(error);
      }

      try {
        let dataset = await util.timeoutFetch('http://localhost:8000/fcu-telemetry', this.refreshTimeMillis - 1);
        dataset.debug_data = debug_data;

        this.dataset = dataset;
      } catch (error) {
        console.log(error);
      }

      try {
        this.graph_data = await util.timeoutFetch('http://localhost:8000/fcu-telemetry/graph', this.refreshTimeMillis - 1);
      } catch (error) {
        console.log(error);
      }
    },
  },
  data() {
    return {
      timer: 0,
      dataset: [],
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
