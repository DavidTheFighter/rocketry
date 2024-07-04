<template>
  <Line :data="chartData()" :options="chartOptions" :key="counter"/>
</template>

<script>
import { Line } from 'vue-chartjs';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  TimeScale,
} from 'chart.js';

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  TimeScale
);

import 'chartjs-adapter-moment';

ChartJS.defaults.backgroundColor = '#222';
ChartJS.defaults.borderColor = '#555';
ChartJS.defaults.color = '#DDD';

Math.fmod = function (a,b) { return Number((a - (Math.floor(a / b) * b)).toPrecision(8)); };

export default {
  name: 'RealtimeLineGraph',
  components: { Line },
  props: {
    dataDescription: {
      type: Array,
    },
    dataset: {
      type: Object,
    },
    displayTimeSeconds: {
      type: Number,
      default: 10.0,
    },
    displayTickInterval: {
      type: Number,
      default: 1.0,
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
    chartRefreshMillis: {
      type: Number,
      default: 33,
    },
    realtimeNumberRefreshMillis: {
      type: Number,
      default: 250,
    },
    dataDecimationModulo: {
      type: Number,
      default: 0.1,
    },
  },
  data() {
    let dataDict = {
      chartLabels: [],
      counter: 0,
      chartOptions: {
        responsive: false,
        animation: false,
        normalized: true,
        spanGaps: 1000.0,
        plugins: {
          legend: {
            position: this.justifyLegend,
            maxWidth: 100,
            labels: {
              usePointStyle: true,
              pointStyle: 'rect',
              font: {
                family: "monospace",
                size: 12,
              },
            },
          },
          tooltip: {
            intersect: false,
          },
          decimation: {
            enabled: true,
          }
        },
        scales: {
          x: {
            type: 'time',
            title: {
              display: this.xTitle != null,
              text: this.xTitle,
              font: {
                size: 16,
              }
            },
            ticks: {
              minRotation: 0,
              maxRotation: 0,
              autoSkip: false,
              callback: (_value, index, ticks) => {
                const numXTicks = this.displayTimeSeconds / this.displayTickInterval;

                if (Math.fmod(index, (ticks.length - 1.0) / numXTicks) < 1.0) {
                  const val =  Math.floor(index / ((ticks.length - 1) / numXTicks));
                  return (val - numXTicks) * this.displayTickInterval;
                } else {
                  return null;
                }
              },
            }
          },
          y: {
            type: 'linear',
            display: true,
            position: 'right',
            title: {
              display: false,
            },
            ticks: {
              minRotation: 0,
              maxRotation: 0,
            },
          },
          y1: {
            type: 'linear',
            display: true,
            position: 'left',
            ticks: {
              display: false,
              minRotation: 0,
              maxRotation: 0,
            },
            title: {
              display: false,
            },
            grid: {
              drawOnChartArea: false, // only want the grid lines for one axis to show up
            },
          },
        },
        elements: {
          point: {
            radius: 0 // default to disabled in all datasets
          }
        },
      },
    };

    if (this.justifyLegend == 'left') {
      dataDict.chartOptions.scales.y1.title = {
        display: false,
      };

      dataDict.chartOptions.scales.y.title = {
        display: this.yTitle != null,
        text: this.yTitle,
        font: {
          size: 16,
        }
      };
    } else {
      dataDict.chartOptions.scales.y.title = {
        display: false,
      };

      dataDict.chartOptions.scales.y1.title = {
        display: this.yTitle != null,
        text: this.yTitle,
        font: {
          size: 16,
        }
      };
    }

    if (this.yrange != null) {
      dataDict.chartOptions.scales.y.min = this.yrange[0];
      dataDict.chartOptions.scales.y.max = this.yrange[1];
      dataDict.chartOptions.scales.y1.min = dataDict.chartOptions.scales.y.min;
      dataDict.chartOptions.scales.y1.max = dataDict.chartOptions.scales.y.max;

      dataDict.chartOptions.scales.y1.ticks = {
        display: true,
      };
    }

    return dataDict;
  },
  created() {
    this.workingDatasets = [];
    for (let i = 0; i < this.dataDescription.length; i++) {
      this.workingDatasets.push([{ x: 0.0, y: 0.0 }, { x: -(this.displayTimeSeconds * 1000), y: 0.0 }]);
      this.chartLabels.push(this.dataDescription[i].name);
    }

    if (this.labelLegend) {
      this.workingDatasets.push([this.workingDatasets[0][0]]); // Invisible middle element
      this.chartLabels.push("");

      for (let i = 0; i < this.dataDescription.length; i++) {
        this.workingDatasets.push([this.workingDatasets[i][0]]);
        this.chartLabels.push("?");
      }
    }
  },
  mounted() {
    this.updateRealtimeValues();
    this.updateish();
    this.fish = 0;
  },
  methods: {
    retrieveField(dataset, fieldName) {
      if (dataset == null || dataset == undefined || fieldName == null || fieldName == undefined) {
        return null;
      }

      const layers = fieldName.split('.');

      let current = dataset;
      for (let i = 0; i < layers.length; i++) {
        if (current[layers[i]] == undefined) {
          return null;
        }

        current = current[layers[i]];
      }

      return current;
    },
    updateish() {
      setTimeout(() => this.updateish(), this.chartRefreshMillis);

      if (this.dataDescription != null && this.dataDescription != undefined) {
        for (let i = 0; i < this.dataDescription.length; i++) {
          const datasetDesc = this.dataDescription[i];

          let data = this.retrieveField(this.dataset, datasetDesc.fieldName);
          if (data == null || data == undefined) {
            continue;
          }

          const scale = datasetDesc.scale ?? 1.0;
          const offset = datasetDesc.offset ?? 0.0;

          let now = new Date();

          // Filter data that's older than the display time
          data = data.filter((pair) => {
            return now - new Date(pair.timestamp) < (this.displayTimeSeconds * 1000);
          });

          // Decimate data if it's too dense (but leave the most recent data below the decimation
          // modulo so there isn't flickering)
          const filetered_data = [data[0]];
          for (let i = 1; i < data.length - 1; i++) {
            if (data[i].timestamp - filetered_data[filetered_data.length - 1].timestamp > this.dataDecimationModulo * 1000) {
              filetered_data.push(data[i]);
            } else if (now - new Date(data[i].timestamp) < this.dataDecimationModulo * 1000) {
              filetered_data.push(data[i]);
            }
          }
          data = filetered_data;

          data = data.map((pair) => {
            if (pair == undefined) {
              return { x: 0.0, y: 0.0 };
            }

            const x = pair.timestamp.getTime() - now.getTime();
            const y = pair.value * scale + offset;
            return { x, y };
          });

          data.unshift({ x: -(this.displayTimeSeconds * 1000), y: data[0]?.y ?? 0.0 });

          // this.bufferLength = data.length;
          this.workingDatasets[i] = [...data];
        }
      }

      this.counter += 1;
    },
    chartData() {
      let chartDatasets = [];
      let maxSize = 0;

      this.dataDescription.forEach((description, index) => {
        chartDatasets.push({
            label: this.chartLabels[index],
            backgroundColor: description.color,
            borderColor: description.color,
            borderWidth: 2,
            data: this.workingDatasets[index],
        });

        maxSize = Math.max(maxSize, description.data?.length);
      });

      if (this.labelLegend) {
        chartDatasets.push({
          label: this.chartLabels[this.dataDescription.length],
          backgroundColor: 'rgba(0, 0, 0, 0.0)',
          borderColor: 'rgba(0, 0, 0, 0.0)',
          borderWidth: 0,
          data: this.workingDatasets[this.dataDescription.length],
        });

        this.dataDescription.forEach((dataset, index) => {
          chartDatasets.push({
              label: this.chartLabels[this.dataDescription.length + 1 + index],
              backgroundColor: dataset.color,
              borderColor: dataset.color,
              borderWidth: 0,
              data: this.workingDatasets[this.dataDescription.length + 1 + index],
          });
        });
      }

      return {
          labels: Array(this.displayTimeSeconds / this.displayTickInterval).fill(null).map((u, i) => {
            const range = this.displayTimeSeconds / this.displayTickInterval;
            return "T" + (-range + (i / (100 / range))).toFixed(1) + "s";
          }),
          datasets: chartDatasets,
      };
    },
    async updateRealtimeValues() {
      setTimeout(() => this.updateRealtimeValues(), this.realtimeNumberRefreshMillis);

      this.dataDescription.forEach((description, index) => {
        const pair = this.workingDatasets[index][this.workingDatasets[index].length - 1];
        const value = pair?.y ?? 0.0;
        let str = `${value.toFixed(this.decimals)}`;
        let intStr = `${Math.floor(value)}`;

        for (let i = 0; i < this.paddingFigs - intStr.length; i++) {
          str = " " + str;
        }

        str = str + (" " + (description.units ?? ""));

        if (this.labelLegend) {
          this.chartLabels[index] = description.name;
          this.chartLabels[this.dataDescription.length + 1 + index] = str;
        } else {
          this.chartLabels[index] = str;
        }
      });
    }
  }
}
</script>
