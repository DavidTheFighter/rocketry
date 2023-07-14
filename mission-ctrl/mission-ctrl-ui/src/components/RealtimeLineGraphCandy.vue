<template>
  <div ref="chart" style="width: 100%; height: 100%"></div>
</template>

<script>
import CandyGraph, {
  createDefaultFont,
  LinearScale,
  OpaqueLineStrip,
  OrthoAxis,
  CartesianCoordinateSystem,
} from "candygraph";

export default {
  name: 'RealtimeLineGraphCandy',
  props: {
    chartRefreshTimeMs: {
      type: Number,
      default: 100,
    },
    historyLengthSeconds: {
      type: Number,
      default: 10.0
    },
  },
  data() {

  },
  methods: {
    render() {
      const st = Date.now();

      const xs = [];
      const ys = [];
      for (let x = 0; x <= 1; x += 0.001) {
        xs.push(x);
        ys.push(0.5 + 0.25 * Math.sin(Date.now() * 1e-3 + x * 2 * Math.PI));
      }

      this.cg.clear([1, 1, 1, 1]);

      const renderables = [
        new OpaqueLineStrip(this.cg, xs, ys, {
          colors: [1, 0.5, 0.0],
          widths: 3,
        }),
        new OrthoAxis(this.cg, this.coords, "x", this.font, {
          labelSide: 1,
          tickOffset: -2.5,
          tickLength: 6,
          tickStep: 0.2,
          labelFormatter: (n) => n.toFixed(1),
        }),
        new OrthoAxis(this.cg, this.coords, "y", this.font, {
          tickOffset: 2.5,
          tickLength: 6,
          tickStep: 0.2,
          labelFormatter: (n) => n.toFixed(1),
        }),
      ];

      this.cg.render(this.coords, this.viewport, renderables);
      this.cg.copyTo(this.viewport, this.canvas);
      renderables.forEach((renderable) => {
        renderable.dispose();
      });

      setTimeout(() => this.render(), Math.max(0, this.chartRefreshTimeMs - (Date.now() - st)));
    }
  },
  async mounted() {
    this.cg = new CandyGraph();
    this.cg.canvas.width = this.$refs.chart.clientWidth;
    this.cg.canvas.height = this.$refs.chart.clientHeight;

    this.xvalues = [];
    for (let x = 0.0; x <= this.historyLengthSeconds; x += this.chartRefreshTimeMs * 1e-3) {
      this.xvalues.push(x);
    }

    const xs = [];
    const ys = [];
    this.x = 0;
    for (let x = 0; x <= 1; x += 0.001) {
      xs.push(x + this.x);
      ys.push(0.5 + 0.25 * Math.sin(Date.now() + x * 2 * Math.PI));
      this.x += 0.001;
    }

    this.viewport = {
      x: 0,
      y: 0,
      width: this.cg.canvas.width,
      height: this.cg.canvas.height,
    };

    this.coords = new CartesianCoordinateSystem(
      this.cg,
      new LinearScale([0, 1], [32, this.viewport.width - 16]),
      new LinearScale([0, 1], [32, this.viewport.height - 16])
    );

    // Load the default Lato font
    this.font = await createDefaultFont(this.cg);

    // Clear the viewport.
    this.cg.clear([1, 1, 1, 1]);

    // Render the a line strip representing the x & y data, and axes.
    const renderables = [
      new OpaqueLineStrip(this.cg, xs, ys, {
        colors: [1, 0.5, 0.0],
        widths: 3,
      }),
      new OrthoAxis(this.cg, this.coords, "x", this.font, {
        labelSide: 1,
        tickOffset: -2.5,
        tickLength: 6,
        tickStep: 0.2,
        labelFormatter: (n) => n.toFixed(1),
      }),
      new OrthoAxis(this.cg, this.coords, "y", this.font, {
        tickOffset: 2.5,
        tickLength: 6,
        tickStep: 0.2,
        labelFormatter: (n) => n.toFixed(1),
      }),
    ];

    this.cg.render(this.coords, this.viewport, renderables);
    this.canvas = this.cg.copyTo(this.viewport);
    this.$refs.chart.appendChild(this.canvas);
    renderables.forEach((renderable) => {
        renderable.dispose();
      });

    this.render();
  }
}
</script>

<style>

</style>