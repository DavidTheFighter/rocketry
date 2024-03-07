<template>
  <div>
    <p v-if="this.title" id="title">{{ this.title }}</p>
      <div v-for="[lalert, ralert] in splitColumnsAlerts" :key="lalert.name" class="row">
        <div v-if="lalert != null" class="row" style="width: 50%">
          <div class="stateTitle" >
            {{ severityBox(lalert.severity) }} {{ lalert.alert ?? "??" }}
          </div>
        </div>
        <div v-if="ralert != null" class="row" style="width: 50%">
          <div class="stateTitle" >
            {{ severityBox(ralert.severity) }} {{ ralert.alert ?? "??" }}
          </div>
        </div>
      </div>
  </div>
</template>

<script>
export default {
  name: "AlertDisplay",
  props: {
    alerts: {
      type: Object,
      required: true,
    },
    title: {
      type: String,
      required: false,
    }
  },
  computed: {
    splitColumnsAlerts() {
      if (this.alerts == null || this.alerts == undefined) {
        return [];
      }

      let arr = [];
      for (let i = 0; i < this.alerts.length; i += 2) {
        if (i >= this.alerts.length - 1) {
          arr.push([this.alerts[i], null]);
          break;
        }

        arr.push([this.alerts[i], this.alerts[i + 1]]);
      }

      return arr;
    },
  },
  methods: {
    severityBox(severity) {
      if (severity == 0) {
        return "🟨";
      } else if (severity == 1) {
        return "🟥";
      } else {
        return "🟦";
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
