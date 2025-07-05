<!-- Create src/components/MusicLibrary.vue -->
<template>
    <div class="music-library">
      <div class="library-header">
        <button @click="$emit('navigate', 'menu')" class="back-button">
          ← Back to Menu
        </button>
        <h2>Music Library</h2>
        <button @click="selectMusicFolder" class="folder-button">
          📁 Select Music Folder
        </button>
      </div>
  
      <div v-if="isScanning" class="scanning-overlay">
        <div class="scanning-content">
          <div class="scanning-spinner"></div>
          <p>Scanning music library...</p>
        </div>
      </div>
  
      <div v-else-if="musicFiles.length === 0" class="no-music">
        <div class="no-music-content">
          <span class="no-music-icon">🎵</span>
          <h3>No Music Found</h3>
          <p>Select a folder containing your music files to get started!</p>
          <button @click="selectMusicFolder" class="select-folder-btn">
            Select Music Folder
          </button>
        </div>
      </div>
  
      <div v-else class="music-list">
        <div class="list-header">
          <div class="search-bar">
            <input 
              v-model="searchQuery" 
              placeholder="Search music..." 
              class="search-input"
            />
          </div>
          <div class="sort-options">
            <select v-model="sortBy" class="sort-select">
              <option value="filename">Filename</option>
              <option value="title">Title</option>
              <option value="artist">Artist</option>
              <option value="duration">Duration</option>
            </select>
          </div>
        </div>
  
        <div class="music-grid">
          <div 
            v-for="file in filteredMusic" 
            :key="file.path"
            @click="selectMusic(file)"
            class="music-item"
            :class="{ 'selected': selectedFile?.path === file.path }"
          >
            <div class="music-info">
              <h4 class="music-title">{{ file.title || file.filename }}</h4>
              <p class="music-artist">{{ file.artist || 'Unknown Artist' }}</p>
              <p class="music-album">{{ file.album || 'Unknown Album' }}</p>
            </div>
            <div class="music-meta">
              <span class="duration">{{ formatDuration(file.duration) }}</span>
              <span class="file-size">{{ formatFileSize(file.file_size) }}</span>
            </div>
          </div>
        </div>
  
        <div v-if="selectedFile" class="selected-music-panel">
          <div class="selected-info">
            <h3>{{ selectedFile.title || selectedFile.filename }}</h3>
            <p>{{ selectedFile.artist || 'Unknown Artist' }} - {{ selectedFile.album || 'Unknown Album' }}</p>
          </div>
          <button @click="playSelectedMusic" class="play-selected-btn">
            Play This Track
          </button>
        </div>
      </div>
    </div>
  </template>
  
  <script lang="ts">
  import { defineComponent, ref, computed, onMounted } from 'vue';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  
  interface MusicFile {
    path: string;
    filename: string;
    title?: string;
    artist?: string;
    album?: string;
    duration?: number;
    file_size: number;
  }
  
  export default defineComponent({
    name: 'MusicLibrary',
    emits: ['navigate', 'play-music'],
    setup(_, { emit }) {
      const musicFiles = ref<MusicFile[]>([]);
      const selectedFile = ref<MusicFile | null>(null);
      const isScanning = ref(false);
      const searchQuery = ref('');
      const sortBy = ref('filename');
      const musicFolderPath = ref('');
  
      const filteredMusic = computed(() => {
        let filtered = musicFiles.value;
  
        // Apply search filter
        if (searchQuery.value) {
          const query = searchQuery.value.toLowerCase();
          filtered = filtered.filter(file => 
            (file.title?.toLowerCase().includes(query)) ||
            (file.artist?.toLowerCase().includes(query)) ||
            (file.album?.toLowerCase().includes(query)) ||
            file.filename.toLowerCase().includes(query)
          );
        }
  
        // Apply sorting
        filtered.sort((a, b) => {
          switch (sortBy.value) {
            case 'title':
              return (a.title || a.filename).localeCompare(b.title || b.filename);
            case 'artist':
              return (a.artist || '').localeCompare(b.artist || '');
            case 'duration':
              return (a.duration || 0) - (b.duration || 0);
            default:
              return a.filename.localeCompare(b.filename);
          }
        });
  
        return filtered;
      });
  
      const selectMusicFolder = async () => {
        try {
          const result = await open({
            directory: true,
            multiple: false,
            title: 'Select Music Folder'
          });
  
          if (result) {
            musicFolderPath.value = result as string;
            await scanMusicFolder();
          }
        } catch (error) {
          console.error('Error selecting folder:', error);
        }
      };
  
      const scanMusicFolder = async () => {
        if (!musicFolderPath.value) return;
  
        isScanning.value = true;
        try {
          const files = await invoke<MusicFile[]>('scan_music_folder', {
            folderPath: musicFolderPath.value
          });
          musicFiles.value = files;
          console.log(`Found ${files.length} music files`);
        } catch (error) {
          console.error('Error scanning music folder:', error);
          musicFiles.value = [];
        } finally {
          isScanning.value = false;
        }
      };
  
      const selectMusic = (file: MusicFile) => {
        selectedFile.value = file;
      };
  
      const playSelectedMusic = async () => {
        if (!selectedFile.value) return;
  
        try {
          // Load the music file data
          const audioData = await invoke<number[]>('load_music_file', {
            filePath: selectedFile.value.path
          });
  
          // Emit to parent component to start the game
          emit('play-music', {
            file: selectedFile.value,
            audioData
          });
        } catch (error) {
          console.error('Error loading music file:', error);
        }
      };
  
      const formatDuration = (seconds?: number): string => {
        if (!seconds) return '--:--';
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins}:${secs.toString().padStart(2, '0')}`;
      };
  
      const formatFileSize = (bytes: number): string => {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
      };
  
      // Load saved music folder on mount
      onMounted(() => {
        const savedPath = localStorage.getItem('musicFolderPath');
        if (savedPath) {
          musicFolderPath.value = savedPath;
          scanMusicFolder();
        }
      });
  
      // Save music folder path
      const saveMusicFolderPath = () => {
        if (musicFolderPath.value) {
          localStorage.setItem('musicFolderPath', musicFolderPath.value);
        }
      };
  
      return {
        musicFiles,
        selectedFile,
        isScanning,
        searchQuery,
        sortBy,
        filteredMusic,
        selectMusicFolder,
        selectMusic,
        playSelectedMusic,
        formatDuration,
        formatFileSize
      };
    }
  });
  </script>
  
  <style scoped>
  .music-library {
    padding: 20px;
    max-width: 1200px;
    margin: 0 auto;
    min-height: 100vh;
  }
  
  .library-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 30px;
    padding: 20px;
    background-color: #2a2a2a;
    border-radius: 12px;
  }
  
  .back-button, .folder-button {
    padding: 10px 20px;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 14px;
    transition: all 0.3s ease;
  }
  
  .back-button {
    background-color: #555;
    color: white;
  }
  
  .folder-button {
    background-color: #646cff;
    color: white;
  }
  
  .back-button:hover, .folder-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }
  
  .scanning-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.8);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }
  
  .scanning-content {
    text-align: center;
    color: white;
  }
  
  .scanning-spinner {
    width: 50px;
    height: 50px;
    border: 4px solid #333;
    border-top: 4px solid #646cff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin: 0 auto 20px;
  }
  
  .no-music {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 400px;
  }
  
  .no-music-content {
    text-align: center;
    color: #ccc;
  }
  
  .no-music-icon {
    font-size: 64px;
    margin-bottom: 20px;
  }
  
  .select-folder-btn {
    padding: 12px 24px;
    background-color: #646cff;
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 16px;
    margin-top: 20px;
  }
  
  .list-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    gap: 20px;
  }
  
  .search-input {
    padding: 10px 15px;
    border: 1px solid #444;
    border-radius: 8px;
    background-color: #333;
    color: white;
    font-size: 14px;
    min-width: 300px;
  }
  
  .sort-select {
    padding: 10px 15px;
    border: 1px solid #444;
    border-radius: 8px;
    background-color: #333;
    color: white;
    font-size: 14px;
  }
  
  .music-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 20px;
    margin-bottom: 20px;
  }
  
  .music-item {
    padding: 20px;
    background-color: #2a2a2a;
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.3s ease;
    border: 2px solid transparent;
  }
  
  .music-item:hover {
    background-color: #333;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }
  
  .music-item.selected {
    border-color: #646cff;
    background-color: #353545;
  }
  
  .music-title {
    font-size: 16px;
    font-weight: bold;
    color: white;
    margin-bottom: 5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .music-artist, .music-album {
    font-size: 14px;
    color: #ccc;
    margin-bottom: 3px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .music-meta {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: #888;
    margin-top: 10px;
  }
  
  .selected-music-panel {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 20px;
    background-color: #2a2a2a;
    border-top: 2px solid #646cff;
    display: flex;
    justify-content: space-between;
    align-items: center;
    z-index: 100;
  }
  
  .selected-info h3 {
    margin: 0 0 5px 0;
    color: white;
  }
  
  .selected-info p {
    margin: 0;
    color: #ccc;
  }
  
  .play-selected-btn {
    padding: 15px 30px;
    background-color: #646cff;
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 16px;
    font-weight: bold;
    transition: all 0.3s ease;
  }
  
  .play-selected-btn:hover {
    background-color: #5a5fd7;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(100, 108, 255, 0.4);
  }
  
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
  </style>