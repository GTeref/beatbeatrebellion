<template>
  <div class="app">
    <MainMenu 
      v-if="currentView === 'menu'" 
      @navigate="handleNavigation"
    />
    
    <MusicLibrary 
      v-else-if="currentView === 'play'" 
      @navigate="handleNavigation"
      @play-music="handlePlayMusic"
    />
    
    <Settings 
      v-else-if="currentView === 'settings'" 
      @navigate="handleNavigation"
    />
    
    <RhythmGame 
      v-else-if="currentView === 'game'" 
      :selected-music="selectedMusic"
      @navigate="handleNavigation"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import RhythmGame from './components/RhythmGame.vue';
import MainMenu from './components/MainMenu.vue';
import MusicLibrary from './components/MusicLibrary.vue';
import Settings from './components/Settings.vue';

export default defineComponent({
  name: 'App',
  components: {
    RhythmGame,
    MainMenu,
    MusicLibrary,
    Settings,
  },

  setup() {
    const currentView = ref('menu');
    const selectedMusic = ref(null);

    const handleNavigation = (view: string) => {
      currentView.value = view;
    };

    const handlePlayMusic = (musicData: any) => {
      selectedMusic.value = musicData;
      currentView.value = 'game';
    };

    return {
      currentView,
      selectedMusic,
      handleNavigation,
      handlePlayMusic
    };
  },
});
</script>

<style>
body {
  margin: 0;
  padding: 0;
  background-color: #1a1a1a;
  color: #fff;
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.app {
  min-height: 100vh;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #1a1a2e 100%);
}

/* Global scrollbar styling */
::-webkit-scrollbar {
  width: 8px;
}

::-webkit-scrollbar-track {
  background: #2a2a2a;
}

::-webkit-scrollbar-thumb {
  background: #646cff;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: #5a5fd7;
}
</style>