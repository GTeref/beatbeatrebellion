<!-- Create src/components/Settings.vue -->
<template>
    <div class="settings">
      <div class="settings-container">
        <div class="settings-header">
          <button @click="$emit('navigate', 'menu')" class="back-button">
            ← Back to Menu
          </button>
          <h2>Settings</h2>
        </div>
  
        <div class="settings-content">
          <div class="setting-group">
            <h3>Audio Settings</h3>
            <div class="setting-item">
              <label>Master Volume:</label>
              <input type="range" min="0" max="100" v-model="masterVolume" />
              <span>{{ masterVolume }}%</span>
            </div>
            <div class="setting-item">
              <label>Hit Sound Volume:</label>
              <input type="range" min="0" max="100" v-model="hitSoundVolume" />
              <span>{{ hitSoundVolume }}%</span>
            </div>
          </div>
  
          <div class="setting-group">
            <h3>Game Settings</h3>
            <div class="setting-item">
              <label>Default Timing Offset:</label>
              <input type="range" min="-1" max="1" step="0.01" v-model="defaultOffset" />
              <span>{{ defaultOffset.toFixed(2) }}s</span>
            </div>
            <div class="setting-item">
              <label>Show FPS:</label>
              <input type="checkbox" v-model="showFPS" />
            </div>
          </div>
  
          <div class="setting-group">
            <h3>Controls</h3>
            <div class="setting-item">
              <label>Lane 1 Key:</label>
              <input type="text" v-model="lane1Key" maxlength="1" />
            </div>
            <div class="setting-item">
              <label>Lane 2 Key:</label>
              <input type="text" v-model="lane2Key" maxlength="1" />
            </div>
            <div class="setting-item">
              <label>Lane 3 Key:</label>
              <input type="text" v-model="lane3Key" maxlength="1" />
            </div>
            <div class="setting-item">
              <label>Lane 4 Key:</label>
              <input type="text" v-model="lane4Key" maxlength="1" />
            </div>
          </div>
  
          <div class="settings-actions">
            <button @click="saveSettings" class="save-button">Save Settings</button>
            <button @click="resetSettings" class="reset-button">Reset to Default</button>
          </div>
        </div>
      </div>
    </div>
  </template>
  
  <script lang="ts">
  import { defineComponent, ref, onMounted } from 'vue';
  
  export default defineComponent({
    name: 'Settings',
    emits: ['navigate'],
    setup() {
      const masterVolume = ref(100);
      const hitSoundVolume = ref(50);
      const defaultOffset = ref(0);
      const showFPS = ref(false);
      const lane1Key = ref('a');
      const lane2Key = ref('s');
      const lane3Key = ref('k');
      const lane4Key = ref('l');
  
      const saveSettings = () => {
        const settings = {
          masterVolume: masterVolume.value,
          hitSoundVolume: hitSoundVolume.value,
          defaultOffset: defaultOffset.value,
          showFPS: showFPS.value,
          controls: {
            lane1: lane1Key.value,
            lane2: lane2Key.value,
            lane3: lane3Key.value,
            lane4: lane4Key.value
          }
        };
  
        localStorage.setItem('gameSettings', JSON.stringify(settings));
        alert('Settings saved!');
      };
  
      const resetSettings = () => {
        masterVolume.value = 100;
        hitSoundVolume.value = 50;
        defaultOffset.value = 0;
        showFPS.value = false;
        lane1Key.value = 'a';
        lane2Key.value = 's';
        lane3Key.value = 'k';
        lane4Key.value = 'l';
      };
  
      const loadSettings = () => {
        const saved = localStorage.getItem('gameSettings');
        if (saved) {
          const settings = JSON.parse(saved);
          masterVolume.value = settings.masterVolume || 100;
          hitSoundVolume.value = settings.hitSoundVolume || 50;
          defaultOffset.value = settings.defaultOffset || 0;
          showFPS.value = settings.showFPS || false;
          lane1Key.value = settings.controls?.lane1 || 'a';
          lane2Key.value = settings.controls?.lane2 || 's';
          lane3Key.value = settings.controls?.lane3 || 'k';
          lane4Key.value = settings.controls?.lane4 || 'l';
        }
      };
  
      onMounted(() => {
        loadSettings();
      });
  
      return {
        masterVolume,
        hitSoundVolume,
        defaultOffset,
        showFPS,
        lane1Key,
        lane2Key,
        lane3Key,
        lane4Key,
        saveSettings,
        resetSettings
      };
    }
  });
  </script>
  
  <style scoped>
  .settings {
    padding: 20px;
    max-width: 800px;
    margin: 0 auto;
    min-height: 100vh;
  }
  
  .settings-container {
    background-color: #2a2a2a;
    border-radius: 12px;
    padding: 30px;
  }
  
  .settings-header {
    display: flex;
    align-items: center;
    gap: 20px;
    margin-bottom: 30px;
    padding-bottom: 20px;
    border-bottom: 1px solid #444;
  }
  
  .back-button {
    padding: 10px 20px;
    background-color: #555;
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.3s ease;
  }
  
  .back-button:hover {
    background-color: #666;
  }
  
  .setting-group {
    margin-bottom: 30px;
  }
  
  .setting-group h3 {
    color: #646cff;
    margin-bottom: 15px;
    font-size: 18px;
  }
  
  .setting-item {
    display: flex;
    align-items: center;
    gap: 15px;
    margin-bottom: 15px;
    padding: 10px;
    background-color: #333;
    border-radius: 8px;
  }
  
  .setting-item label {
    min-width: 180px;
    color: #ccc;
  }
  
  .setting-item input[type="range"] {
    flex: 1;
    max-width: 200px;
  }
  
  .setting-item input[type="text"] {
    width: 40px;
    padding: 5px;
    text-align: center;
    background-color: #444;
    border: 1px solid #555;
    border-radius: 4px;
    color: white;
  }
  
  .setting-item span {
    color: #ccc;
    font-family: monospace;
    min-width: 60px;
  }
  
  .settings-actions {
    display: flex;
    gap: 20px;
    justify-content: center;
    margin-top: 30px;
    padding-top: 20px;
    border-top: 1px solid #444;
  }
  
  .save-button, .reset-button {
    padding: 12px 24px;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 16px;
    transition: all 0.3s ease;
  }
  
  .save-button {
    background-color: #646cff;
    color: white;
  }
  
  .save-button:hover {
    background-color: #5a5fd7;
  }
  
  .reset-button {
    background-color: #ff5555;
    color: white;
  }
  
  .reset-button:hover {
    background-color: #ff3333;
  }
  </style>