<!-- filepath: c:\Users\giova\Documents\Projects\beatbeatrebellion\beatbeatrebellion\src\components\RhythmGame.vue -->
<template>
    <div class="game-container">
      <!-- File upload section -->
      <div v-if="!audioFile" class="upload-section">
        <h2>BeatBeatRebellion</h2>
        <p>Upload a music file to start playing!</p>
        <input type="file" accept="audio/*" @change="handleFileUpload" />
      </div>
      
      <!-- Game section -->
      <div v-else class="game-section">
        <div class="game-header">
          <div class="score">Score: {{ score }}</div>
          <button @click="resetGame" class="reset-btn">Reset</button>
        </div>
        
        <div class="game-area" ref="gameArea">
          <!-- Lane indicators -->
          <div class="lanes-container">
            <div v-for="(lane, index) in lanes" :key="index" class="lane">
              <div class="lane-key">{{ lane.key.toUpperCase() }}</div>
            </div>
          </div>
          
          <!-- Hit line -->
          <div class="hit-line"></div>
          
          <!-- Notes (will be dynamically added) -->
          <div 
            v-for="note in activeNotes" 
            :key="note.id" 
            class="note" 
            :class="{ 'hit': note.hit }"
            :style="{ 
              top: `${note.position}px`, 
              left: `${(note.lane * 25) + 12.5}%` 
            }">
          </div>
        </div>
        
        <div class="audio-controls">
          <button @click="startGame" :disabled="isPlaying">Start</button>
          <button @click="pauseGame" :disabled="!isPlaying">Pause</button>
        </div>
      </div>
    </div>
  </template>
  
  <script lang="ts">
  import { defineComponent, ref, reactive, onMounted, onBeforeUnmount } from 'vue';
  
  export default defineComponent({
    name: 'RhythmGame',
    setup() {
      const audioFile = ref<File | null>(null);
      const audioContext = ref<AudioContext | null>(null);
      const audioBuffer = ref<AudioBuffer | null>(null);
      const audioSource = ref<AudioBufferSourceNode | null>(null);
      const analyser = ref<AnalyserNode | null>(null);
      const isPlaying = ref(false);
      const gameArea = ref<HTMLElement | null>(null);
      const score = ref(0);
      const activeNotes = ref<any[]>([]);
      const noteId = ref(0);
      
      // Define lanes (A, S, K, L keys)
      const lanes = reactive([
        { key: 'a', active: false },
        { key: 's', active: false },
        { key: 'k', active: false },
        { key: 'l', active: false }
      ]);
      
      // Animation frame ID for game loop
      let animationFrameId: number;
      // Timestamp for controlling note generation
      let lastNoteTime = 0;
      
      const handleFileUpload = async (event: Event) => {
        const input = event.target as HTMLInputElement;
        if (input.files && input.files.length > 0) {
          audioFile.value = input.files[0];
          await loadAudio();
        }
      };
      
      const loadAudio = async () => {
        if (!audioFile.value) return;
        
        // Create audio context
        audioContext.value = new (window.AudioContext || (window as any).webkitAudioContext)();
        
        // Read file as ArrayBuffer
        const arrayBuffer = await audioFile.value.arrayBuffer();
        
        // Decode audio data
        audioBuffer.value = await audioContext.value.decodeAudioData(arrayBuffer);
        
        // Create analyser node
        analyser.value = audioContext.value.createAnalyser();
        analyser.value.fftSize = 2048;
      };
      
      const startGame = () => {
        if (!audioContext.value || !audioBuffer.value || isPlaying.value) return;
        
        // Reset game state
        activeNotes.value = [];
        score.value = 0;
        
        // Create source node
        audioSource.value = audioContext.value.createBufferSource();
        audioSource.value.buffer = audioBuffer.value;
        
        // Connect nodes
        audioSource.value.connect(analyser.value!);
        analyser.value!.connect(audioContext.value.destination);
        
        // Start audio
        audioSource.value.start(0);
        isPlaying.value = true;
        
        // Start game loop
        lastNoteTime = Date.now();
        gameLoop();
      };
      
      const pauseGame = () => {
        if (!isPlaying.value || !audioSource.value) return;
        
        audioSource.value.stop();
        isPlaying.value = false;
        cancelAnimationFrame(animationFrameId);
      };
      
      const resetGame = () => {
        if (isPlaying.value) {
          pauseGame();
        }
        audioFile.value = null;
        audioBuffer.value = null;
        activeNotes.value = [];
        score.value = 0;
      };
      
      const gameLoop = () => {
        // Analyze audio and generate notes
        if (isPlaying.value && analyser.value) {
          const bufferLength = analyser.value.frequencyBinCount;
          const dataArray = new Uint8Array(bufferLength);
          analyser.value.getByteFrequencyData(dataArray);
          
          // Simple peak detection (can be improved)
          const currentTime = Date.now();
          if (currentTime - lastNoteTime > 500) { // Limit note generation rate
            // Calculate average volume across frequency ranges for each lane
            const laneVolumes = [
              calculateVolumeForRange(dataArray, 0, 100),      // Bass (Lane 0)
              calculateVolumeForRange(dataArray, 100, 300),    // Low-mid (Lane 1)
              calculateVolumeForRange(dataArray, 300, 800),    // Mid (Lane 2)
              calculateVolumeForRange(dataArray, 800, 2000)    // High (Lane 3)
            ];

          
            // Find the highest volume and generate a note if it's above threshold
            const maxVolume = Math.max(...laneVolumes);
            const threshold = 130; // Adjust based on testing
            
            if (maxVolume > threshold) {
              const laneIndex = laneVolumes.indexOf(maxVolume);
              generateNote(laneIndex);
              lastNoteTime = currentTime;
            }
          }
          
          // Update notes positions
          updateNotes();
        }
        
        animationFrameId = requestAnimationFrame(gameLoop);
      };
      
      const calculateVolumeForRange = (dataArray: Uint8Array, startFreq: number, endFreq: number): number => {
        const startIndex = Math.floor(startFreq / 22050 * dataArray.length);
        const endIndex = Math.min(Math.floor(endFreq / 22050 * dataArray.length), dataArray.length - 1);
        
        let sum = 0;
        for (let i = startIndex; i <= endIndex; i++) {
          sum += dataArray[i];
        }
        
        return sum / (endIndex - startIndex + 1);
      };
      
      const generateNote = (laneIndex: number) => {
        activeNotes.value.push({
          id: noteId.value++,
          lane: laneIndex,
          position: 0,
          hit: false
        });
      };
      
      const updateNotes = () => {
        if (!gameArea.value) return;
        
        const hitLinePosition = gameArea.value.clientHeight - 100;
        const hitThreshold = 30; // Acceptable range to hit a note
        
        // Move notes down
        activeNotes.value.forEach(note => {
          if (!note.hit) {
            note.position += 3; // Adjust speed as needed
          }
        });
        
        // Remove notes that went too far
        activeNotes.value = activeNotes.value.filter(note => {
          // If note passed the hit line by too much, remove it
          if (note.position > gameArea.value!.clientHeight && !note.hit) {
            return false;
          }
          return true;
        });
      };
      
      // Handle key presses
      const handleKeyDown = (event: KeyboardEvent) => {
        if (!isPlaying.value) return;
        
        const key = event.key.toLowerCase();
        const laneIndex = lanes.findIndex(lane => lane.key === key);
        
        if (laneIndex !== -1) {
          lanes[laneIndex].active = true;
          
          // Check if there's a note to hit
          if (!gameArea.value) return;
          
          const hitLinePosition = gameArea.value.clientHeight - 100;
          const hitThreshold = 30; // Acceptable range to hit a note
          
          // Find notes in this lane near the hit line
          const noteToHit = activeNotes.value.find(note => {
            return !note.hit && 
                   note.lane === laneIndex && 
                   Math.abs(note.position - hitLinePosition) < hitThreshold;
          });
          
          if (noteToHit) {
            noteToHit.hit = true;
            score.value += 100;
          }
        }
      };
      
      const handleKeyUp = (event: KeyboardEvent) => {
        const key = event.key.toLowerCase();
        const laneIndex = lanes.findIndex(lane => lane.key === key);
        
        if (laneIndex !== -1) {
          lanes[laneIndex].active = false;
        }
      };
      
      onMounted(() => {
        window.addEventListener('keydown', handleKeyDown);
        window.addEventListener('keyup', handleKeyUp);
      });
      
      onBeforeUnmount(() => {
        window.removeEventListener('keydown', handleKeyDown);
        window.removeEventListener('keyup', handleKeyUp);
        
        if (isPlaying.value) {
          pauseGame();
        }
        
        if (audioContext.value) {
          audioContext.value.close();
        }
      });
      
      return {
        audioFile,
        isPlaying,
        gameArea,
        lanes,
        score,
        activeNotes,
        handleFileUpload,
        startGame,
        pauseGame,
        resetGame
      };
    }
  });
  </script>
  
  <style scoped>
  .game-container {
    width: 100%;
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
    font-family: Arial, sans-serif;
  }
  
  .upload-section {
    text-align: center;
    padding: 50px 0;
  }
  
  .game-section {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  
  .game-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .score {
    font-size: 24px;
    font-weight: bold;
  }
  
  .game-area {
    position: relative;
    width: 100%;
    height: 600px;
    background-color: #1a1a1a;
    border-radius: 8px;
    overflow: hidden;
  }
  
  .lanes-container {
    display: flex;
    height: 100%;
    width: 100%;
  }
  
  .lane {
    flex: 1;
    height: 100%;
    border-right: 1px solid #333;
    position: relative;
  }
  
  .lane:last-child {
    border-right: none;
  }
  
  .lane-key {
    position: absolute;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    width: 40px;
    height: 40px;
    background-color: #3a3a3a;
    color: white;
    display: flex;
    justify-content: center;
    align-items: center;
    border-radius: 4px;
    font-weight: bold;
    z-index: 5;
  }
  
  .hit-line {
    position: absolute;
    bottom: 100px;
    width: 100%;
    height: 4px;
    background-color: #ff5555;
    z-index: 4;
  }
  
  .note {
    position: absolute;
    width: 40px;
    height: 40px;
    background-color: #5555ff;
    border-radius: 50%;
    transform: translateX(-50%);
    z-index: 3;
  }
  
  .note.hit {
    background-color: #55ff55;
    opacity: 0.5;
  }
  
  .audio-controls {
    display: flex;
    gap: 10px;
    justify-content: center;
  }
  
  button {
    padding: 10px 20px;
    background-color: #646cff;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }
  
  button:disabled {
    background-color: #333;
    cursor: not-allowed;
  }
  
  .reset-btn {
    background-color: #ff5555;
  }
  </style>