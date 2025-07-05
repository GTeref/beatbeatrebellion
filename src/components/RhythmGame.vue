<!-- filepath: c:\Users\giova\Documents\Projects\beatbeatrebellion\beatbeatrebellion\src\components\RhythmGame.vue -->
<template>
    <div class="game-container">
      <!-- File upload section -->
      <div v-if="!audioFile" class="upload-section">
        <h2>BeatBeatRebellion</h2>
        <p>Upload a music file to start playing!</p>
        <input type="file" accept="audio/*" @change="handleFileUpload" />
      </div>

      <!--loading screen-->
      <div v-else-if="isLoadingBeatmap" class="loading-section">
        <div class="loading-content">
          <div class="loading-spinner"></div>
          <h3>Analyzing Audio...</h3>
          <p>Generating beatmap for: {{ audioFile?.name }}</p>
          <div class="loading-progress">
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: `${loadingProgress}%` }"></div>
            </div>
            <span class="progress-text">{{ loadingProgress }}%</span>
          </div>
        </div>
      </div>
      
      <!-- Game section -->
      <div v-else-if="selectedMusic" class="game-section">
        <div class="game-header">
          <button @click="goBack" class="back-button">← Back to Library</button>
          <div class="current-track">
            <h3>{{ selectedMusic.file.title || selectedMusic.file.filename }}</h3>
            <p>{{ selectedMusic.file.artist || 'Unknown Artist' }}</p>
          </div>
          <div class="score">Score: {{ score }}</div>

          <div class="timing-controls">
          <label>
            Timing Offset: 
            <input 
              type="range" 
              min="-1.0" 
              max="1.0" 
              step="0.01" 
              v-model.number="audioLatencyOffset"
              :disabled="isPlaying"
            />
            <span>{{ audioLatencyOffset.toFixed(2) }}s</span>
          </label>
        </div>
          <div class="autoplay-toggle">
            <label>
              <input 
                type="checkbox" 
                v-model="autoplay" 
                :disabled="isPlaying"
              /> 
              Autoplay
            </label>
          </div>
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
            :class="{
               'hit': note.hit,
               'holding': note.isHolding,
                'single': note.type === 'single',
                'hold': note.type === 'hold'
            }"
            :style="{ 
              top: `${note.position}px`, 
              left: `${(note.lane * 25) + 12.5}%`,
              height: note.type === 'hold' ? `${note.duration || 0}px` : '40px',
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
  import { invoke } from '@tauri-apps/api/core';
  
  export default defineComponent({
    name: 'RhythmGame',
    props: {
      selectedMusic: {
        type: Object,
        default: null
      }
    },
    emits: ['navigate'],
    setup(props, { emit }) {
      const audioFile = ref<File | null>(null);
      const audioContext = ref<AudioContext | null>(null);
      const audioBuffer = ref<AudioBuffer | null>(null);
      const audioSource = ref<AudioBufferSourceNode | null>(null);
      const audioStartTime = ref(0);
      const analyser = ref<AnalyserNode | null>(null);
      const isPlaying = ref(false);
      const gameArea = ref<HTMLElement | null>(null);
      const score = ref(0);
      const activeNotes = ref<any[]>([]);
      const noteId = ref(0);
      const generatedBeatmap = ref<any[]>([]);
      const beatmapIndex = ref(0);
      const lastBeatTime = ref(0);
      const heldKeys=ref<Set<string>>(new Set());
      const autoplay = ref(false); // Autoplay toggle
      const isLoadingBeatmap = ref(false);
      const loadingProgress = ref(0); // Progress for beatmap generation
      const audioLatencyOffset = ref(0); // Offset for audio latency

      interface Note {
        id: number;
        lane: number;
        position: number;
        hit: boolean;
        type: 'single' | 'hold';
        duration?: number; // For hold notes
        holdProgress?: number; // For hold notes
        isHolding?: boolean; // For hold notes
      }
      
      // Define lanes (A, S, K, L keys)
      const lanes = reactive([
        { key: 'a', active: false },
        { key: 's', active: false },
        { key: 'k', active: false },
        { key: 'l', active: false }
      ]);

      const goBack=()=>{
        if (isPlaying.value) {
          pauseGame();
        }
        emit('navigate', 'play');
      }
      
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
        if (!props.selectedMusic) return;

        try {

          //start loading
          isLoadingBeatmap.value = true;
          loadingProgress.value = 0;

          //placebo progress updating

          // Create audio context
          audioContext.value = new (window.AudioContext || (window as any).webkitAudioContext)();
          loadingProgress.value = 10;
          
          // Read file as ArrayBuffer
          const audioBytes=new Uint8Array(props.selectedMusic.audioData);
          const arrayBuffer = audioBytes.buffer;
          loadingProgress.value = 20;

          const dupArrayBuffer=arrayBuffer.slice(0);
          loadingProgress.value = 30;

          // Decode audio data
          audioBuffer.value = await audioContext.value.decodeAudioData(dupArrayBuffer);
          loadingProgress.value = 40;

          loadingProgress.value = 50;

          console.log(`Generating beatmap for audio file: ${props.selectedMusic.value.name}`);

          const progressInterval = setInterval(() => {
            if (loadingProgress.value < 90) {
              loadingProgress.value += Math.random() * 5;
            }
          }, 200);

          const beatmapPromise=await invoke<number[][]>('analyze_audio', {
            audioData: props.selectedMusic.audioData
          });

          const beatmap=await beatmapPromise;
          clearInterval(progressInterval);

          generatedBeatmap.value = beatmap;
          beatmapIndex.value=0;
          loadingProgress.value = 100;
          

          console.log(`generated beatmap w/ ${beatmap.length} notes`);
          
        } catch (error) {
          console.error('Error generating beatmap:', error);
          // generatedBeatmap.value = [];
        } finally {
          isLoadingBeatmap.value = false;
          loadingProgress.value = 0;
        }

        
        //Create analyser node
        analyser.value = audioContext.value.createAnalyser();
        analyser.value.fftSize = 2048;
      };
      
      const startGame = () => {
        if (!audioContext.value || !audioBuffer.value || isPlaying.value) return;
        
        // Reset game state
        activeNotes.value = [];
        score.value = 0;
        beatmapIndex.value = 0;
        
        // Create source node
        audioSource.value = audioContext.value.createBufferSource();
        audioSource.value.buffer = audioBuffer.value;

        //listen for audio end
        audioSource.value.onended = () => {
          isPlaying.value = false;
          cancelAnimationFrame(animationFrameId);
          console.log('Audio playback ended');
        };
        
        // Connect nodes
        if (analyser.value){
          audioSource.value.connect(analyser.value!);
          analyser.value!.connect(audioContext.value.destination);
        } else {
          audioSource.value.connect(audioContext.value.destination);
        }
        
        audioStartTime.value = audioContext.value.currentTime;

        
        // Start audio
        audioSource.value.start(0);
        isPlaying.value = true;
        
        // Start game loop
        // lastNoteTime = Date.now();
        // lastBeatTime.value = audioContext.value.currentTime;
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
        isLoadingBeatmap.value = false;
        loadingProgress.value = 0;
      };
      
      const gameLoop = () => {
        console.log('Game loop running', { 
          isPlaying: isPlaying.value, 
          beatmapLength: generatedBeatmap.value.length,
          currentIndex: beatmapIndex.value 
        });
        // Analyze audio and generate notes
        if (isPlaying.value && generatedBeatmap.value.length > 0) {
          // Calculate timing based on audio playback
          const currentTime = (audioContext.value!.currentTime - audioStartTime.value) + audioLatencyOffset.value;
          
          // Check if we should generate the next note
          while (beatmapIndex.value < generatedBeatmap.value.length) {
            const nextNote=generatedBeatmap.value[beatmapIndex.value];

            const leadTime=2.0;

            const noteSpawnTime=nextNote.timestamp-leadTime;

            if (currentTime >= noteSpawnTime) {

              const duration=nextNote.note_type === 'hold' ? nextNote.duration*100 : 0;
              generateNote(nextNote.lane, nextNote.note_type, duration);
              beatmapIndex.value++;
              console.log(`Spawning note at audio time: ${currentTime.toFixed(2)}s, note timestamp: ${nextNote.timestamp.toFixed(2)}s`);
            } else {
              break; // No more notes to generate at this time
            }
          }
          
          // Update notes positions
          updateNotes();
        }
        if (isPlaying.value){
          animationFrameId = requestAnimationFrame(gameLoop);
        }
        
      };
      
      const generateNote = (laneIndex: number, noteType: 'single' | 'hold' = 'single', duration: number=0) => {
        // activeNotes.value.push({
        //   id: noteId.value++,
        //   lane: laneIndex,
        //   position: 0,
        //   hit: false
        // });
        const note: Note = {
          id: noteId.value++,
          lane: laneIndex,
          position: 0,
          hit: false,
          type: noteType,
          duration: noteType === 'hold' ? duration : 0,
          holdProgress: 0,
          isHolding: false
        };

        activeNotes.value.push(note);
        console.log(`Generated ${noteType} note in lane ${laneIndex}${noteType === 'hold' ? ` (duration: ${duration}px)` : ''}`);
      };
      
      const updateNotes = () => {
        if (!gameArea.value) return;
        
        const hitLinePosition = gameArea.value.clientHeight - 100;
        const hitThreshold = 30; // Acceptable range to hit a note

        const leadTime = 2.0; // make this global later, same number as in gameLoop
        const noteSpeed = hitLinePosition / (leadTime * 60); // Speed at which notes move down
        
        // Move notes down
        activeNotes.value.forEach(note => {
          if (!note.hit) {
            note.position += noteSpeed; // Adjust speed as needed

            // AUTOPLAY LOGIC
            if (autoplay.value && !note.hit) {
              // Check if note is at the hit line
              if (Math.abs(note.position - hitLinePosition) <= hitThreshold) {
                // Automatically hit the note
                if (note.type === 'single') {
                  note.hit = true;
                  score.value += 100;
                  generateHitSound(note.lane, note.type);
                  
                  // Visual feedback for autoplay
                  lanes[note.lane].active = true;
                  setTimeout(() => {
                    lanes[note.lane].active = false;
                  }, 100);
                  
                } else if (note.type === 'hold' && !note.isHolding) {
                  note.isHolding = true;
                  score.value += 50;
                  generateHitSound(note.lane, note.type);
                  
                  // Keep lane active during hold
                  lanes[note.lane].active = true;
                }
              }
            }

            //hold note logic
            if (note.type === 'hold' && note.isHolding) {
              const noteEnd=note.position + (note.duration || 0);

              if (Math.abs(noteEnd - hitLinePosition) < hitThreshold ||
                  Math.abs(note.position - hitLinePosition) < hitThreshold ||
                  (note.position < hitLinePosition && noteEnd > hitLinePosition)) {
                note.holdProgress = Math.min((note.holdProgress || 0) + 2, note.duration || 0);
                score.value += 2; // Increment score for holding

                if (note.holdProgress >= (note.duration || 0)) {
                  note.hit = true; // Mark as hit when hold is complete
                  // note.isHolding = false; // Stop holding
                  score.value += 100; // Bonus for completing hold
                  generateHoldCompleteSound(note.lane);
                }
              }
            }
          }
        });
        
        // Remove notes that went too far
        activeNotes.value = activeNotes.value.filter(note => {
          // If note passed the hit line by too much, remove it
          if (note.position > gameArea.value!.clientHeight && !note.hit) {
            if (autoplay.value && note.type === 'hold' && note.isHolding) {
              lanes[note.lane].active = false;
            }
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
          heldKeys.value.add(key);
          
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
            generateHitSound(laneIndex, noteToHit.type);
            if (noteToHit.type=='single') {
              noteToHit.hit = true;
              score.value += 100;
            } else if (noteToHit.type=='hold') {
              noteToHit.isHolding = true;
              score.value += 50; // Initial score for hold notes
            }
            
          } else {
            generateMissSound(laneIndex);
          }
        }
      };
      
      const handleKeyUp = (event: KeyboardEvent) => {
        const key = event.key.toLowerCase();
        const laneIndex = lanes.findIndex(lane => lane.key === key);
        
        if (laneIndex !== -1) {
          lanes[laneIndex].active = false;
          heldKeys.value.delete(key);

          activeNotes.value.forEach(note=>{
            if (note.lane===laneIndex && note.type==='hold' && note.isHolding) {
              note.isHolding = false;
              // score.value += 100; // Bonus for holding
            }
          })
        }
      };

      // sound generation logic below
      const generateHitSound = (laneIndex: number, noteType: 'single' | 'hold') => {
        if (!audioContext.value) return;
        
        const frequencies=[220,277,349,440] // A3, C#4, F4, A4
        const frequency = frequencies[laneIndex];

        const oscillator = audioContext.value.createOscillator();
        const gainNode = audioContext.value.createGain();

        oscillator.connect(gainNode);
        gainNode.connect(audioContext.value.destination);

        if (noteType==='single'){
          oscillator.type='sine'
          oscillator.frequency.setValueAtTime(frequency, audioContext.value.currentTime);

          gainNode.gain.setValueAtTime(0, audioContext.value.currentTime);
          gainNode.gain.linearRampToValueAtTime(0.3, audioContext.value.currentTime + 0.01); // Fade out quickly
          gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.value.currentTime + 0.15); // Ensure it goes to zero

          oscillator.start(audioContext.value.currentTime); 
          oscillator.stop(audioContext.value.currentTime + 0.15); // Short sound duration
        } else {
          oscillator.type='sawtooth'
          oscillator.frequency.setValueAtTime(frequency, audioContext.value.currentTime);

          gainNode.gain.setValueAtTime(0, audioContext.value.currentTime);
          gainNode.gain.linearRampToValueAtTime(0.2, audioContext.value.currentTime + 0.05); // Fade in quickly
          gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.value.currentTime + 0.3); // Ensure it goes to zero

          oscillator.start(audioContext.value.currentTime);
          oscillator.stop(audioContext.value.currentTime + 0.3); // Longer sound duration for hold notes

        }
      };

      const generateMissSound = (laneIndex: number) => {
        if (!audioContext.value) return;

        const oscillator = audioContext.value.createOscillator();
        const gainNode = audioContext.value.createGain();

        oscillator.connect(gainNode);
        gainNode.connect(audioContext.value.destination);

        oscillator.type='triangle';
        oscillator.frequency.setValueAtTime(100+(laneIndex*20), audioContext.value.currentTime);

        gainNode.gain.setValueAtTime(0, audioContext.value.currentTime);
        gainNode.gain.linearRampToValueAtTime(0.1, audioContext.value.currentTime + 0.01); // Fade out quickly
        gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.value.currentTime + 0.1); // Ensure it goes to zero

        oscillator.start(audioContext.value.currentTime);
        oscillator.stop(audioContext.value.currentTime + 0.1); // Short sound duration
      };

       // completion sound for hold notes
      const generateHoldCompleteSound = (laneIndex: number) => {
        if (!audioContext.value) return;

        const frequencies = [220, 277, 349, 440];
        const baseFreq = frequencies[laneIndex];

        // Create a chord for completion
        for (let i = 0; i < 3; i++) {
          const oscillator = audioContext.value.createOscillator();
          const gainNode = audioContext.value.createGain();

          oscillator.connect(gainNode);
          gainNode.connect(audioContext.value.destination);

          oscillator.type = 'sine';
          oscillator.frequency.setValueAtTime(baseFreq * (1 + i * 0.25), audioContext.value.currentTime);
          
          gainNode.gain.setValueAtTime(0, audioContext.value.currentTime);
          gainNode.gain.linearRampToValueAtTime(0.15, audioContext.value.currentTime + 0.02);
          gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.value.currentTime + 0.4);
          
          oscillator.start(audioContext.value.currentTime);
          oscillator.stop(audioContext.value.currentTime + 0.4);
        }
      };
      
      onMounted(() => {
        window.addEventListener('keydown', handleKeyDown);
        window.addEventListener('keyup', handleKeyUp);
        if (props.selectedMusic) {
          loadAudio();
        }
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
        resetGame,
        autoplay,
        isLoadingBeatmap,
        loadingProgress,
        audioStartTime,
        audioLatencyOffset,
        goBack,
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
    background-color: #5555ff;
    border-radius: 20px;
    transform: translateX(-50%);
    z-index: 3;
    transition: background-color 0.1s ease;
  }

  .note.single {
    height: 40px;
    border-radius: 50%;
  }

  .note.hold {
    border-radius: 20px 20px 8px 8px;
    background: linear-gradient(180deg, #5555ff 0%, #3333dd 100%);
    border: 2px solid #7777ff;
    min-height: 40px;
  }

  .note.hold::after {
    content: '';
    position: absolute;
    top: 100%;
    left: 50%;
    transform: translateX(-50%);
    width: 8px;
    background: linear-gradient(180deg, #3333dd 0%, #1111bb 100%);
    border-radius: 0 0 4px 4px;
  }

  .note.hit {
    background-color: #55ff55;
    opacity: 0.8;
  }

  .note.holding {
    background-color: #ffff55;
    box-shadow: 0 0 10px #ffff55;
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

  .autoplay-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .autoplay-toggle label {
    display: flex;
    align-items: center;
    gap: 5px;
    cursor: pointer;
    font-size: 14px;
  }

  .autoplay-toggle input[type="checkbox"] {
    width: 16px;
    height: 16px;
  }

  .autoplay-toggle input[type="checkbox"]:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .loading-section {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 400px;
    text-align: center;
  }

  .loading-content {
    max-width: 400px;
    padding: 40px;
    background-color: #2a2a2a;
    border-radius: 12px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  }

  .loading-spinner {
    width: 50px;
    height: 50px;
    border: 4px solid #333;
    border-top: 4px solid #646cff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin: 0 auto 20px;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .loading-content h3 {
    margin: 0 0 10px 0;
    color: #fff;
    font-size: 24px;
  }

  .loading-content p {
    margin: 0 0 20px 0;
    color: #ccc;
    font-size: 14px;
    word-break: break-all;
  }

  .loading-progress {
    display: flex;
    flex-direction: column;
    gap: 10px;
    align-items: center;
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background-color: #333;
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #646cff 0%, #8b5cf6 100%);
    transition: width 0.3s ease;
    border-radius: 4px;
  }

  .progress-text {
    font-size: 12px;
    color: #ccc;
    font-weight: bold;
  }

  .timing-controls {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    font-size: 12px;
  }

  .timing-controls input[type="range"] {
    width: 100px;
    margin: 0 8px;
  }

  .timing-controls span {
    font-family: monospace;
    color: #ccc;
  }
  </style>