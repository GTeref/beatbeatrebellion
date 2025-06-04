<template>
  <div class="container">
    <RhythmGame />
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import RhythmGame from './components/RhythmGame.vue';

export default defineComponent({
  name: 'App',
  components: {
    RhythmGame,
  },
});
</script>

<style>
body {
  margin: 0;
  padding: 0;
  background-color: #242424;
  color: #fff;
  font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.container {
  margin: 0 auto;
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
}
</style>