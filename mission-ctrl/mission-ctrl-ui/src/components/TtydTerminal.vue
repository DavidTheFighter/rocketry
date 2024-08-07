<template>
  <iframe id="terminal" src="http://127.0.0.1:8001"></iframe>
</template>

<script>

export default {
  name: 'TtydTerminal',
  data() {
    return {
    };
  },
};
</script>

<style scoped>
#terminal {
  width: 100%;
  height: 100%;
}
</style>