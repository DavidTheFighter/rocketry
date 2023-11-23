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
  Legend
} from 'chart.js';

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend
);

ChartJS.defaults.backgroundColor = '#222';
ChartJS.defaults.borderColor = '#555';
ChartJS.defaults.color = '#DDD';

Math.fmod = function (a,b) { return Number((a - (Math.floor(a / b) * b)).toPrecision(8)); };

export default {
  name: 'RealtimeLineGraphChartjs',
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
    realtimeNumberRefreshMillis: {
      type: Number,
      default: 250,
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
        spanGaps: true,
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
            threshold: 20,
          }
        },
        scales: {
          x: {
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
    this.bufferLength = 100;

    this.workingDatasets = [];
    for (let i = 0; i < this.dataDescription.length; i++) {
      const workingDataset = [];
      for (let j = 0; j < this.bufferLength; j++) {
        workingDataset.push(0.0);
      }

      this.workingDatasets.push(workingDataset);
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
    this.fish = 0;
  },
  watch: {
    dataset: {
      handler() {
        this.updateish();
      },
      immediate: true,
    },
  },
  methods: {
    updateish() {
      if (this.dataDescription != null && this.dataDescription != undefined) {
        for (let i = 0; i < this.dataDescription.length; i++) {
          const datasetDesc = this.dataDescription[i];
          if (datasetDesc.dataName == undefined) {
            continue;
          }

          if (this.dataset == undefined || this.dataset == null) {
            continue;
          }

          if (this.dataset[datasetDesc.dataName] == undefined) {
            continue;
          }

          let data = this.dataset[datasetDesc.dataName];
          if (datasetDesc.dataIndex != undefined && datasetDesc.dataIndex != null) {
            data = data[datasetDesc.dataIndex];
          }

          if (data == null || data == undefined || data.length == 0) {
            continue;
          }

          const scale = datasetDesc.scale ?? 1.0;
          const offset = datasetDesc.offset ?? 0.0;

          data = data.map((x) => x * scale + offset);

          this.bufferLength = data.length;
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
          labels: Array(this.bufferLength).fill(null).map((u, i) => {
            const range = this.displayTimeSeconds / this.displayTickInterval;
            return "T" + (-range + (i / (this.bufferLength / range))).toFixed(1) + "s";
          }),
          datasets: chartDatasets,
      };
    },
    async updateRealtimeValues() {
      setTimeout(() => this.updateRealtimeValues(), this.realtimeNumberRefreshMillis);

      this.dataDescription.forEach((description, index) => {
        const value = this.workingDatasets[index][this.workingDatasets[index].length - 1];
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
