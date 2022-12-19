<template>
  <div>
    <p id="title">Software State</p>
    <div class="row">
      <div class="stateColumn">
        <div class="row">
          <div class="columnLeft">
            <p class="stateTitle" v-for="state in leftColumnStates" :key="state.name">
              {{ state.name }}
            </p>
          </div>
          <div class="columnRight">
            <p class="stateValue" v-for="state in leftColumnStates" :key="state.name">
              {{ stateValueText(state) }}
            </p>
          </div>
        </div>
      </div>
      <div class="stateColumn">
        <div class="row">
          <div class="columnLeft">
            <p class="stateTitle" v-for="state in rightColumnStates" :key="state.name">
              {{ state.name }}
            </p>
          </div>
          <div class="columnRight">
            <p class="stateValue" v-for="state in rightColumnStates" :key="state.name">
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
        return state.value + " " + state.units;
      } else {
        return state.value;
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

.row {
  display: flex;
}

.stateColumn {
  flex: 50%;
  max-width: 50%;
}

.columnLeft {
  flex: 50%;
  max-width: 50%;
  margin-left: 0%;
}

.columnRight {
  flex: 50%;
  max-width: 50%;
}
</style>