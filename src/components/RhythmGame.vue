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
      const generatedBeatmap = ref<any[]>([]);
      const beatmapIndex = ref(0);
      const lastBeatTime = ref(0);
      const heldKeys=ref<Set<string>>(new Set());

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

        const dupArrayBuffer=arrayBuffer.slice(0);
        
        // Decode audio data
        audioBuffer.value = await audioContext.value.decodeAudioData(dupArrayBuffer);

        const audioBytes=new Uint8Array(arrayBuffer);

        try{
          console.log(`Generating beatmap for audio file: ${audioFile.value.name}`);
          const beatmap=await invoke<number[][]>('analyze_audio', {
            audioData: Array.from(audioBytes)
          });

          generatedBeatmap.value = beatmap;
          beatmapIndex.value=0;

          console.log(`generated beatmap w/ ${beatmap.length} notes`);
        } catch (error) {
          console.error('Error generating beatmap:', error);
          // generatedBeatmap.value = [];
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
        
        
        // Start audio
        audioSource.value.start(0);
        isPlaying.value = true;
        
        // Start game loop
        // lastNoteTime = Date.now();
        lastBeatTime.value = audioContext.value.currentTime;
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
        console.log('Game loop running', { 
          isPlaying: isPlaying.value, 
          beatmapLength: generatedBeatmap.value.length,
          currentIndex: beatmapIndex.value 
        });
        // Analyze audio and generate notes
        if (isPlaying.value && generatedBeatmap.value.length > 0) {
          // Calculate timing based on audio playback
          const currentTime = audioContext.value?.currentTime || 0;
          
          // Check if we should generate the next note
          while (beatmapIndex.value < generatedBeatmap.value.length) {
            const nextNote=generatedBeatmap.value[beatmapIndex.value];

            const noteSpawnTime=nextNote.timestamp-3.0;

            if (currentTime >= noteSpawnTime) {

              const duration=nextNote.note_type === 'hold' ? nextNote.duration*100 : 0;
              generateNote(nextNote.lane, nextNote.note_type, duration);
              beatmapIndex.value++;
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
      
      const calculateVolumeForRange = (dataArray: Uint8Array, startFreq: number, endFreq: number): number => {
        const startIndex = Math.floor(startFreq / 22050 * dataArray.length);
        const endIndex = Math.min(Math.floor(endFreq / 22050 * dataArray.length), dataArray.length - 1);
        
        let sum = 0;
        for (let i = startIndex; i <= endIndex; i++) {
          sum += dataArray[i];
        }
        
        return sum / (endIndex - startIndex + 1);
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
        
        // Move notes down
        activeNotes.value.forEach(note => {
          if (!note.hit) {
            note.position += 3; // Adjust speed as needed

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
                }
              }
            }
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
            if (noteToHit.type=='single') {
              noteToHit.hit = true;
              score.value += 100;
            } else if (noteToHit.type=='hold') {
              noteToHit.isHolding = true;
              score.value += 50; // Initial score for hold notes
            }
            
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
  </style>