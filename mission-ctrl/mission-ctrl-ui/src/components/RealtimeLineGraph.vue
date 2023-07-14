<template>
  <div style="width: 100%; height: 100%" ref="graph"></div>
</template>

<script>
import TimeChart from 'timechart/core';
import { lineChart } from 'timechart/plugins/lineChart';
import { d3Axis } from 'timechart/plugins/d3Axis';
// import { legend } from 'timechart/plugins/legend';
import { crosshair } from 'timechart/plugins/crosshair';
import { nearestPoint } from 'timechart/plugins/nearestPoint';

Math.fmod = function (a,b) { return Number((a - (Math.floor(a / b) * b)).toPrecision(8)); };

export default {
  name: 'RealtimeLineGraph',
  components: { },
  props: {
    datasets: {
      type: Array,
    },
    yrange: {
      type: Array,
      default: null
    },
    xTitle: {
      type: String,
      default: null
    },
    yTitle: {
      type: String,
      default: null
    },
    decimals: {
      type: Number,
      default: 1
    },
    paddingFigs: {
      type: Number,
      default: 0
    },
    labelLegend: {
      type: Boolean,
      default: true,
    },
    justifyLegend: {
      type: String,
      default: "right",
    },
    realtimeNumberRefreshMillis: {
      type: Number,
      default: 250,
    },
    numXTicks: {
      type: Number,
      default: 10.0,
    },
    scaleXTicks: {
      type: Number,
      default: 1.0,
    },
  },
  mounted() {
    const canvas = this.$refs.graph;

    this.series = [];
    this.datasets.forEach((dataset) => {
      this.series.push({
        data: [],
        data_len: 0,
        name: dataset.name,
      });
    });

    this.chart = new TimeChart(canvas, {
      series: this.series,
      plugins: {
        lineChart,
        d3Axis,
        // legend,
        crosshair,
        nearestPoint,
      }
    });
    this.start = Date.now();
    this.updateLineGraph();
  },
  // watch: {
  //   datasets: {
  //     handler() {
  //       this.updateLineGraph();
  //     },
  //     immediate: true,
  //   },
  // },
  methods: {
    async updateRealtimeValues() {
      setTimeout(() => this.updateRealtimeValues(), this.realtimeNumberRefreshMillis);

      if (this.dataset == null || this.dataset == undefined) {
        return;
      }

      // this.realtimeValues = this.datasets.map((dataset) => {
      //   if (dataset.data == null || dataset.data == undefined || dataset.data.length == 0) {
      //     return "0";
      //   }

      //   let str = `${dataset.data[dataset.data.length - 1].toFixed(this.decimals)}`;
      //   let intStr = `${Math.floor(dataset.data[dataset.data.length - 1])}`;

      //   for (let i = 0; i < this.paddingFigs - intStr.length; i++) {
      //     str = " " + str;
      //   }

      //   return str;
      // });
    },
    async updateLineGraph() {
      if (this.series == undefined || this.series == null) {
        return;
      }

      setTimeout(() => this.updateLineGraph(), 33);

      let updated = false;
      this.series.forEach((line, index) => {
        const dataset = this.datasets[index];
        if (dataset.data == null) {
          return;
        }

        if (dataset.data.length == 0) {
          return;
        }
        line.data.push({ x: line.data_len, y: dataset.data[dataset.data.length - 1] });
        line.data_len += 1;
        updated = true;
      });

      if (updated) {
        this.chart.update();
      }

      console.log(Date.now() - this.start);
      this.start = Date.now();
    }
  }
}
</script>
