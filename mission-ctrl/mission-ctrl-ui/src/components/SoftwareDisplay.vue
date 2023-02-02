<template>
  <div>
    <p id="title">Software State</p>
    <div class="row">
      <div class="stateColumn" style="margin-left: 5%">
        <div class="row">
          <div class="columnLeft">
            <p class="stateTitle" v-for="state in leftColumnStates" :key="state.name">
              {{ state.name ?? "??" }}
            </p>
          </div>
          <div class="columnRight">
            <p v-for="state in leftColumnStates" :key="state.name" :class="stateStyleClass(state)">
              {{ stateValueText(state) }}
            </p>
          </div>
        </div>
      </div>
      <div class="stateColumn">
        <div class="row">
          <div class="columnLeft">
            <p class="stateTitle" v-for="state in rightColumnStates" :key="state.name">
              {{ state.name ?? "??" }}
            </p>
          </div>
          <div class="columnRight">
            <p v-for="state in rightColumnStates" :key="state.name" :class="stateStyleClass(state)">
              {{ stateValueText(state) }}
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: "SoftwareDisplay",
  props: {
    states: {
      type: Array,
      required: true
    },
  },
  computed: {
    leftColumnStates() {
      return this.states.filter((_value, index) => index % 2 == 0);
    },
    rightColumnStates() {
      return this.states.filter((_value, index) => index % 2 == 1);
    },
  },
  methods: {
    stateValueText(state) {
      if (state.units) {
        return (state.value ?? "??") + " " + state.units;
      } else {
        return state.value ?? "??";
      }
    },
    stateStyleClass(state) {
      if (state.value == 0 || state.value == undefined) {
        return "stateValue bad";
      } else {
        return "stateValue";
      }
    }
  }
};
</script>

<style scoped>
#title {
  text-align: center;
  font-family: monospace;
  font-size: 1.75em;
}

.stateTitle {
  text-align: left;
  font-family: monospace;
  font-size: 1.25em;
}

.stateValue {
  text-align: center;
  font-family: monospace;
  font-size: 1.25em;
  color: dodgerblue
}

.stateValue.bad {
  text-align: center;
  font-family: monospace;
  font-size: 1.25em;
  color: red;
}

.row {
  display: flex;
}

.stateColumn {
  flex: 50%;
}

.columnLeft {
  flex: 50%;
}

.columnRight {
  flex: 50%;
}
</style>