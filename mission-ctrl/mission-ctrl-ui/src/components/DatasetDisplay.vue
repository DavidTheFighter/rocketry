<template>
  <div>
    <p v-if="this.title" id="title">{{ this.title }}</p>
    <div v-if="!this.singleColumn" class="row">
      <div class="stateColumn" style="margin-left: 5%">
        <div v-for="state in leftColumnStates" :key="state.name" class="row">
          <div class="columnLeft">
            <div class="stateTitle">
              {{ state.name ?? "??" }}
            </div>
          </div>
          <div class="columnRight">
            <div :class="stateStyleClass(state)">
              {{ stateValueText(state) }}
            </div>
          </div>
        </div>
      </div>
      <div class="stateColumn">
        <div v-for="state in rightColumnStates" :key="state.name" class="row">
          <div class="columnLeft">
            <div class="stateTitle">
              {{ state.name ?? "??" }}
            </div>
          </div>
          <div class="columnRight">
            <div :class="stateStyleClass(state)">
              {{ stateValueText(state) }}
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
      if (state.badValue != null && state.badValue != undefined) {
        if (state.badValue) {
          return "stateValue bad";
        } else {
          return "stateValue";
        }
      } else if (state.value == 0 || state.value == undefined ) {
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
  font-size: 1.25em;
  margin-top: 5px;
  margin-bottom: 5px;
}

.stateValue {
  text-align: center;
  font-family: monospace;
  font-size: 1.25em;
  color: dodgerblue;
  margin-top: 5px;
  margin-bottom: 5px;
}

.stateValue.bad {
  text-align: center;
  font-family: monospace;
  font-size: 1.25em;
  color: red;
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