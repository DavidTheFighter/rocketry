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
        "valve": (args) => this.postCommand(args),
        "testvalve": (args) => this.postCommand(args),
        "press": (args) => this.postCommand(args),
        "fp": (args) => this.commandAlias("press", args),
        "depress": (args) => this.postCommand(args),
        "dfp": (args) => this.commandAlias("depress", args),
        "fire": (args) => this.postCommand(args),
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