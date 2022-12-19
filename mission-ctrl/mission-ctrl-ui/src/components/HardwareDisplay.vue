<template>
  <div>
    <p id="title">Hardware State</p>
    <div class="row">
      <div class="valveColumn">
        <div class="row">
          <div class="columnLeft">
            <p class="valveTitle" v-for="valve in leftColumnValves" :key="valve.name">
              {{valve.name}}
            </p>
          </div>
          <div class="columnRight">
            <p v-for="valve in leftColumnValves" :key="valve.name" :class="valveStateCSSClass(valve)">
              {{valve.data.state}}
            </p>
          </div>
        </div>
      </div>
      <div class="valveColumn">
        <div class="row">
          <div class="columnLeft">
            <p class="valveTitle" v-for="valve in rightColumnValves" :key="valve.name">
              {{valve.name}}
            </p>
          </div>
          <div class="columnRight">
            <p v-for="valve in rightColumnValves" :key="valve.name" :class="valveStateCSSClass(valve)">
              {{valve.data.state}}
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: "HardwareDisplay",
  props: {
    valves: {
      type: Array,
      required: true
    }
  },
  computed: {
    leftColumnValves() {
      return this.valves.filter((value, index) => index % 2 == 0);
    },
    rightColumnValves() {
      return this.valves.filter((value, index) => index % 2 == 1);
    },
  },
  methods: {
    valveStateCSSClass(valve) {
      if (valve.data.in_default_state) {
        return "valveState normal";
      } else {
        return "valveState actuated";
      }
    },
  }
};
</script>

<style scoped>
#title {
  text-align: center;
  font-family: monospace;
  font-size: 1.75em;
}

.valveTitle {
  text-align: left;
  margin-left: 10%;
  font-family: monospace;
  font-size: 1.25em;
}

.valveState {
  text-align: center;
  font-family: monospace;
  font-size: 1.25em;
}

.valveState.normal {
  color: green;
}

.valveState.actuated {
  color: red;
}

.row {
  display: flex;
}

.valveColumn {
  flex: 50%;
  max-width: 50%;
}

.columnLeft {
  flex: 75%;
  max-width: 75%;
}

.columnRight {
  flex: 25%;
  max-width: 25%;
}
</style>