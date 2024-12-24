<template>
  <div>
    <p v-if="this.title" id="title">{{ this.title }}</p>
    <div v-if="!this.singleColumn">
      <div v-for="[lstate, rstate] in splitColumnStates" :key="lstate.name" class="row">
        <div v-if="lstate != null" class="row" style="width: 50%">
          <div class="columnLeft">
            <div class="stateTitle">
              {{ lstate.name ?? "??" }}
            </div>
          </div>
          <div class="columnRight">
            <div :class="stateStyleClass(lstate)">
              {{ stateValueText(lstate) }}
            </div>
          </div>
        </div>
        <div v-if="rstate != null" class="row" style="width: 50%">
          <div class="columnLeft">
            <div class="stateTitle">
              {{ rstate.name ?? "??" }}
            </div>
          </div>
          <div class="columnRight">
            <div :class="stateStyleClass(rstate)">
              {{ stateValueText(rstate) }}
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-else class="row">
      <div style="margin-left: 5%; width: 100%; ">
        <div v-for="state in this.states" :key="state.name" class="row">
          <div class="columnLeft">
            <div class="stateTitle"> {{ state.name ?? "??" }} </div>
          </div>
          <div class="columnRight">
            <div :class="stateStyleClass(state)"> {{ stateValueText(state) }} </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: "DatasetDisplay",
  props: {
    states: {
      type: Array,
      required: true,
    },
    title: {
      type: String,
      required: false,
    },
    singleColumn: {
      type: Boolean,
      default: false,
      required: false,
    },
  },
  computed: {
    splitColumnStates() {
      let arr = [];
      for (let i = 0; i < this.states.length; i += 2) {
        if (i >= this.states.length - 1) {
          arr.push([this.states[i], null]);
          break;
        }

        arr.push([this.states[i], this.states[i + 1]]);
      }

      return arr;
    },
    leftColumnStates() {
      return this.states.filter((_value, index) => index % 2 == 0);
    },
    rightColumnStates() {
      return this.states.filter((_value, index) => index % 2 == 1);
    },
  },
  methods: {
    stateValueText(state) {
      let value = "??";
      if (state?.value != null && state?.value != undefined) {
        value = state.value;
      } else if (state?.lastValue != null && state?.lastValue != undefined && state.lastValue.length > 0) {
        value = state.lastValue[state.lastValue.length - 1]['value'] ?? "??";
      }

      if (state?.units) {
        return value + " " + state.units;
      } else {
        return value;
      }
    },
    stateStyleClass(state) {
      let value = null;
      if (state?.value != null && state?.value != undefined) {
        value = state.value;
      } else if (state?.lastValue != null && state?.lastValue != undefined && state.lastValue.length > 0) {
        value = state.lastValue[state.lastValue.length - 1]['value'] ?? "??";
      }

      if (state?.badValue != null && state?.badValue != undefined) {
        if (state.badValue) {
          return "stateValue bad";
        } else {
          return "stateValue";
        }
      } else if (value == 0 || value == undefined || value == null) {
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
  margin-top: 5px;
  margin-bottom: 5px;
}

.stateTitle {
  text-align: left;
  font-family: monospace;
  font-size: 1.15em;
  margin-top: 5px;
  margin-bottom: 5px;
}

.stateValue {
  text-align: center;
  font-family: monospace;
  font-size: 1.15em;
  color: #0BF;
  margin-top: 5px;
  margin-bottom: 5px;
}

.stateValue.bad {
  text-align: center;
  font-family: monospace;
  font-size: 1.15em;
  color: #D00;
  margin-top: 5px;
  margin-bottom: 5px;
}

.row {
  display: flex;
}

.stateColumn {
  flex: 50%;
}

.stateSingleColumn {
  flex: 100%;
}

.columnLeft {
  flex: 50%;
}

.columnRight {
  flex: 50%;
}
</style>