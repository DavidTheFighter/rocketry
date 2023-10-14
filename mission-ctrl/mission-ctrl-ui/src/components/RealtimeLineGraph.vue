<template>
  <Line :data="chartData" :options="chartOptions" />
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
  Math.fmod = function (a,b) { return Number((a - (Math.floor(a / b) * b)).toPrecision(8)); };
  export default {
    name: 'RealtimeLineGraph',
    components: { Line },
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
    computed: {
      chartData() {
        let chartDatasets = [];
        let maxSize = 0;
        this.datasets.forEach(dataset => {
          if (dataset.data == null) {
            dataset.data = [];
            for (let i = 0; i < 100; i++) {
              dataset.data.push(Math.sin(i / 5) * 50 + 50);
            }
          }
          chartDatasets.push({
              label: dataset.name,
              backgroundColor: dataset.color,
              borderColor: dataset.color,
              borderWidth: 2,
              data: dataset.data,
          });
          maxSize = Math.max(maxSize, dataset.data?.length);
        });
        chartDatasets.push({
          label: '',
          backgroundColor: 'rgba(0, 0, 0, 0.0)',
          borderColor: 'rgba(0, 0, 0, 0.0)',
          borderWidth: 0,
          data: [this.datasets[0].data[0]],
        });
        this.datasets.forEach((dataset, index) => {
          chartDatasets.push({
              label: (this.realtimeValues == null ? "?" : this.realtimeValues[index]) + (" " + (dataset.units ?? "")),
              backgroundColor: dataset.color,
              borderColor: dataset.color,
              borderWidth: 0,
              data: [dataset.data[0]],
          });
        });
        return {
            labels: Array(maxSize).fill(null).map((u, i) => {
              return "T" + (-10 + (i / (maxSize / 10.0))).toFixed(1) + "s";
            }),
            datasets: chartDatasets,
        };
      },
    },
    data() {
      let dataDict = {
        realtimeValues: null,
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
                  if (Math.fmod(index, (ticks.length - 1.0) / this.numXTicks) < 1.0) {
                    const val =  Math.floor(index / ((ticks.length - 1) / this.numXTicks));
                    return (val - this.numXTicks) * this.scaleXTicks;
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
          }
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
    mounted() {
      this.updateRealtimeValues();
    },
    methods: {
      async updateRealtimeValues() {
        setTimeout(() => this.updateRealtimeValues(), this.realtimeNumberRefreshMillis);
        this.realtimeValues = this.datasets.map((dataset) => {
          let str = `${dataset.data[dataset.data.length - 1].toFixed(this.decimals)}`;
          let intStr = `${Math.floor(dataset.data[dataset.data.length - 1])}`;
          for (let i = 0; i < this.paddingFigs - intStr.length; i++) {
            str = " " + str;
          }
          return str;
        });
      }
    }
  }
  </script>
  