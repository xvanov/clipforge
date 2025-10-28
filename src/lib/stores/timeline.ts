import { writable, derived, type Writable } from 'svelte/store';
import type { Track, TimelineClip } from '$lib/types/timeline';

// Timeline store - holds all tracks and clips
export const timeline: Writable<Track[]> = writable([]);

// Playhead position (in seconds)
export const playheadPosition: Writable<number> = writable(0);

// Playing state
export const isPlaying: Writable<boolean> = writable(false);

// Zoom level for timeline (pixels per second)
export const timelineZoom: Writable<number> = writable(50);

// Derived: Total timeline duration
export const timelineDuration = derived(timeline, ($tracks) => {
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

// Helper functions
export function addTrack(track: Track) {
  timeline.update((tracks) => [...tracks, track]);
}

export function addClipToTrack(trackId: string, clip: TimelineClip) {
  timeline.update((tracks) =>
    tracks.map((track) => {
      if (track.id === trackId) {
        return {
          ...track,
          clips: [...track.clips, clip],
        };
      }
      return track;
    })
  );
}

export function removeClipFromTimeline(clipId: string) {
  timeline.update((tracks) =>
    tracks.map((track) => ({
      ...track,
      clips: track.clips.filter((c) => c.id !== clipId),
    }))
  );
}

export function play() {
  isPlaying.set(true);
}

export function pause() {
  isPlaying.set(false);
}

export function seek(position: number) {
  playheadPosition.set(position);
}
