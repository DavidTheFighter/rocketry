<template>
  <vue-command :commands="commands" />
</template>

<script>
import VueCommand, { createStdout, textFormatter } from "vue-command";
import "vue-command/dist/vue-command.css";

export default {
  name: "RocketTerminal",
  components: {
    VueCommand,
  },
  data () {
    return {
      commands: {
        "sv-valve": (args) => this.postCommand(args),
        "sv" : (args) => this.commandAlias("sv-valve", args),
        "test-sv-valve": (args) => this.postCommand(args),
        "test-sv": (args) => this.commandAlias("test-sv-valve", args),
        "test-spark": (args) => this.postCommand(args),
        "test-fire-igniter": (args) => this.postCommand(args),
        "fuel-press": (args) => this.postCommand(args),
        "fp": (args) => this.commandAlias("fuel-press", args),
        "fuel-depress": (args) => this.postCommand(args),
        "dfp": (args) => this.commandAlias("fuel-depress", args),
        "fuel-idle": (args) => this.postCommand(args),
        "fi": (args) => this.commandAlias("fuel-idle", args),
      },
    };
  },
  methods: {
    async postCommand(args) {
      try {
        const response = await fetch(`http://localhost:8000/commands/${args[0]}`, {
          method: "POST",
          headers: {
            "Accept": "application/json",
            "Content-Type": "application/json" ,
            "Access-Control-Allow-Origin": "*",
            "Access-Control-Allow-Methods": "POST",
          },
          body: JSON.stringify(args),
        });

        if (!response.ok) {
          return createStdout(textFormatter(`${args[0]} is unimplemented on the server!`));
        }

        const data = await response.json();

        return createStdout(textFormatter(data.text_response));
      } catch (ex) {
        console.error(ex);
        return createStdout(textFormatter(`An error happened: ${ex}`));
      }
    },
    async commandAlias(fullCommand, args) {
      args[0] = fullCommand;
      return this.postCommand(args);
    }
  }
};
</script>