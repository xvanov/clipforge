import { writable, get, type Writable } from 'svelte/store';
import type { RecordingSession, RecordingSources } from '$lib/types/recording';

// Recording state interface
export interface RecordingState {
  isRecording: boolean;
  isPreparing: boolean;
  currentSession: RecordingSession | null;
  recordingDuration: number;
  error: string | null;

  // Recording sources and configuration
  sources: RecordingSources | null;
  selectedScreenId: string | null;
  selectedCameraId: string | null;
  selectedMicrophoneId: string | null;
  recordingType: 'screen' | 'webcam' | 'screen_webcam';
  includeAudio: boolean;
  includeMicrophone: boolean;
  resolution: string;
  fps: number;
}

// Initial state
const initialState: RecordingState = {
  isRecording: false,
  isPreparing: false,
  currentSession: null,
  recordingDuration: 0,
  error: null,
  sources: null,
  selectedScreenId: null,
  selectedCameraId: null,
  selectedMicrophoneId: null,
  recordingType: 'screen',
  includeAudio: true,
  includeMicrophone: true,
  resolution: '1920x1080',
  fps: 30,
};

// Create the store
export const recordingStore: Writable<RecordingState> = writable(initialState);

// Helper functions for updating recording state
export function setRecordingSources(sources: RecordingSources) {
  recordingStore.update((state) => {
    const newState = { ...state, sources };

    // Auto-select first available sources if none selected
    if (!state.selectedScreenId && sources.screens.length > 0) {
      newState.selectedScreenId = sources.screens[0].id;
    }
    if (!state.selectedCameraId && sources.cameras.length > 0) {
      newState.selectedCameraId = sources.cameras[0].id;
    }
    if (!state.selectedMicrophoneId && sources.microphones.length > 0) {
      newState.selectedMicrophoneId = sources.microphones[0].id;
    }

    return newState;
  });
}

export function startPreparingRecording() {
  recordingStore.update((state) => ({
    ...state,
    isPreparing: true,
    error: null,
  }));
}

export function startRecordingSession(session: RecordingSession) {
  recordingStore.update((state) => ({
    ...state,
    isRecording: true,
    isPreparing: false,
    currentSession: session,
    recordingDuration: 0,
    error: null,
  }));
}

export function updateRecordingDuration(duration: number) {
  recordingStore.update((state) => ({
    ...state,
    recordingDuration: duration,
  }));
}

export function stopRecordingSession() {
  recordingStore.update((state) => ({
    ...state,
    isRecording: false,
    isPreparing: false,
    currentSession: null,
    recordingDuration: 0,
  }));
}

export function setRecordingError(error: string) {
  recordingStore.update((state) => ({
    ...state,
    error,
    isPreparing: false,
  }));
}

export function clearRecordingError() {
  recordingStore.update((state) => ({
    ...state,
    error: null,
  }));
}

export function updateRecordingConfig(config: Partial<RecordingState>) {
  recordingStore.update((state) => ({
    ...state,
    ...config,
  }));
}

// Getters
export function getCurrentSession(): RecordingSession | null {
  return get(recordingStore).currentSession;
}

export function isCurrentlyRecording(): boolean {
  return get(recordingStore).isRecording;
}
