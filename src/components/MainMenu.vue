<template>
    <div class="main-menu">
      <div class="menu-container">
        <h1 class="game-title">BeatBeatRebellion</h1>
        <div class="menu-buttons">
          <button @click="$emit('navigate', 'play')" class="menu-button play-button">
            <span class="button-icon">🎵</span>
            Play
          </button>
          <button @click="$emit('navigate', 'settings')" class="menu-button settings-button">
            <span class="button-icon">⚙️</span>
            Settings
          </button>
          <button @click="exitGame" class="menu-button exit-button">
            <span class="button-icon">🚪</span>
            Exit
          </button>
        </div>
      </div>
    </div>
  </template>
  
  <script lang="ts">
  import { defineComponent } from 'vue';
  import { exit } from '@tauri-apps/plugin-process';
  
  export default defineComponent({
    name: 'MainMenu',
    emits: ['navigate'],
    setup() {
      const exitGame = async () => {
        await exit(0);
      };
  
      return {
        exitGame
      };
    }
  });
  </script>
  
  <style scoped>
  .main-menu {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #1a1a2e 100%);
  }
  
  .menu-container {
    text-align: center;
    padding: 40px;
    background-color: rgba(42, 42, 42, 0.9);
    border-radius: 20px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .game-title {
    font-size: 48px;
    font-weight: bold;
    color: #ffffff;
    margin-bottom: 40px;
    text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.5);
    background: linear-gradient(135deg, #646cff, #8b5cf6);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  
  .menu-buttons {
    display: flex;
    flex-direction: column;
    gap: 20px;
    align-items: center;
  }
  
  .menu-button {
    display: flex;
    align-items: center;
    gap: 15px;
    padding: 15px 30px;
    font-size: 18px;
    font-weight: bold;
    border: none;
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.3s ease;
    min-width: 200px;
    justify-content: center;
    text-transform: uppercase;
    letter-spacing: 1px;
  }
  
  .button-icon {
    font-size: 24px;
  }
  
  .play-button {
    background: linear-gradient(135deg, #646cff, #8b5cf6);
    color: white;
  }
  
  .play-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 25px rgba(100, 108, 255, 0.4);
  }
  
  .settings-button {
    background: linear-gradient(135deg, #ffa500, #ff6b35);
    color: white;
  }
  
  .settings-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 25px rgba(255, 165, 0, 0.4);
  }
  
  .exit-button {
    background: linear-gradient(135deg, #ff5555, #ff3333);
    color: white;
  }
  
  .exit-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 25px rgba(255, 85, 85, 0.4);
  }
  
  .menu-button:active {
    transform: translateY(0);
  }
  </style>