import { writable, derived } from 'svelte/store';
import type { Track, TimelineClip } from '$lib/types/timeline';
import { invoke } from '@tauri-apps/api/tauri';

// Timeline State Interface
interface TimelineState {
  tracks: Track[];
  currentTime: number;
  isPlaying: boolean;
  zoom: number;
}

// Create main timeline store
const createTimelineStore = () => {
  const { subscribe, set, update } = writable<TimelineState>({
    tracks: [],
    currentTime: 0,
    isPlaying: false,
    zoom: 50,
  });

  return {
    subscribe,
    set,
    update,

    // Add clip to timeline via Tauri command
    addClipToTimeline: async (
      mediaClipId: string,
      trackId: string,
      startTime: number,
      inPoint: number,
      outPoint: number
    ) => {
      try {
        const timelineClip = await invoke<TimelineClip>('add_clip_to_timeline', {
          mediaClipId,
          trackId,
          startTime,
          inPoint,
          outPoint,
        });

        update((state) => ({
          ...state,
          tracks: state.tracks.map((track) => {
            if (track.id === trackId) {
              return {
                ...track,
                clips: [...track.clips, timelineClip],
              };
            }
            return track;
          }),
        }));

        return timelineClip;
      } catch (error) {
        console.error('Failed to add clip to timeline:', error);
        throw error;
      }
    },

    // Update timeline clip
    updateClip: async (
      clipId: string,
      updates: {
        startTime?: number;
        inPoint?: number;
        outPoint?: number;
        trackId?: string;
      }
    ) => {
      try {
        // Convert camelCase to snake_case for Rust
        const rustUpdates = {
          start_time: updates.startTime,
          in_point: updates.inPoint,
          out_point: updates.outPoint,
          track_id: updates.trackId,
        };

        const updatedClip = await invoke<TimelineClip>('update_timeline_clip', {
          clipId,
          updates: rustUpdates,
        });

        update((state) => {
          // Find the track containing this clip and only update that track
          const trackIndex = state.tracks.findIndex((track) =>
            track.clips.some((clip) => clip.id === clipId)
          );

          if (trackIndex === -1) return state;

          const updatedTracks = [...state.tracks];
          updatedTracks[trackIndex] = {
            ...updatedTracks[trackIndex],
            clips: updatedTracks[trackIndex].clips.map((clip) =>
              clip.id === clipId ? updatedClip : clip
            ),
          };

          return {
            ...state,
            tracks: updatedTracks,
          };
        });

        return updatedClip;
      } catch (error) {
        console.error('Failed to update clip:', error);
        throw error;
      }
    },

    // Split clip at specified time
    splitClip: async (clipId: string, splitTime: number) => {
      try {
        const result = await invoke<{ clip_before: TimelineClip; clip_after: TimelineClip }>(
          'split_timeline_clip',
          {
            clipId,
            splitTime,
          }
        );

        update((state) => ({
          ...state,
          tracks: state.tracks.map((track) => ({
            ...track,
            clips: track.clips.flatMap((clip) => {
              if (clip.id === clipId) {
                return [result.clip_before, result.clip_after];
              }
              return clip;
            }),
          })),
        }));

        return result;
      } catch (error) {
        console.error('Failed to split clip:', error);
        throw error;
      }
    },

    // Delete clip from timeline
    deleteClip: async (clipId: string) => {
      try {
        await invoke('delete_timeline_clip', { clipId });

        update((state) => ({
          ...state,
          tracks: state.tracks.map((track) => ({
            ...track,
            clips: track.clips.filter((clip) => clip.id !== clipId),
          })),
        }));
      } catch (error) {
        console.error('Failed to delete clip:', error);
        throw error;
      }
    },

    // Create new track
    createTrack: async (name: string, trackType: 'main' | 'overlay') => {
      try {
        const track = await invoke<Track>('create_track', {
          name,
          trackType,
        });

        update((state) => {
          const newState = {
            ...state,
            tracks: [...state.tracks, track],
          };
          return newState;
        });

        return track;
      } catch (error) {
        console.error('Failed to create track:', error);
        throw error;
      }
    },

    // Playback controls
    play: () => {
      update((state) => ({ ...state, isPlaying: true }));
    },

    pause: () => {
      update((state) => ({ ...state, isPlaying: false }));
    },

    seek: (time: number) => {
      update((state) => ({ ...state, currentTime: time }));
    },

    setZoom: (zoom: number) => {
      update((state) => ({ ...state, zoom }));
    },
  };
};

export const timelineStore = createTimelineStore();

// Derived stores for convenience
export const tracks = derived(timelineStore, ($state) => $state.tracks);
export const currentTime = derived(timelineStore, ($state) => $state.currentTime);
export const isPlaying = derived(timelineStore, ($state) => $state.isPlaying);
export const timelineZoom = derived(timelineStore, ($state) => $state.zoom);

// Derived: Total timeline duration
export const timelineDuration = derived(tracks, ($tracks) => {
  if ($tracks.length === 0) return 0;

  return Math.max(
    ...$tracks.map((track) => {
      if (track.clips.length === 0) return 0;
      return Math.max(
        ...track.clips.map((clip) => clip.start_time + (clip.out_point - clip.in_point))
      );
    })
  );
});
