import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import {
  timelineStore,
  tracks,
  currentTime,
  isPlaying,
  timelineZoom,
  timelineDuration,
} from './timeline';
import type { Track, TimelineClip } from '$lib/types/timeline';

// Mock Tauri API
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(),
}));

describe('Timeline Store', () => {
  beforeEach(() => {
    // Reset store to initial state
    timelineStore.set({
      tracks: [],
      currentTime: 0,
      isPlaying: false,
      zoom: 50,
    });
  });

  it('should start with empty timeline', () => {
    const $tracks = get(tracks);
    expect($tracks).toEqual([]);
  });

  it('should start with playhead at 0', () => {
    const $currentTime = get(currentTime);
    expect($currentTime).toBe(0);
  });

  it('should start in paused state', () => {
    const $isPlaying = get(isPlaying);
    expect($isPlaying).toBe(false);
  });

  it('should start with default zoom level', () => {
    const $zoom = get(timelineZoom);
    expect($zoom).toBe(50);
  });

  it('should add a track to the timeline', () => {
    const mockTrack: Track = {
      id: 'track-1',
      name: 'Main Track',
      type: 'main',
      order: 0,
      clips: [],
      visible: true,
      locked: false,
      volume: 1.0,
    };

    timelineStore.update((state) => ({
      ...state,
      tracks: [...state.tracks, mockTrack],
    }));

    const $tracks = get(tracks);
    expect($tracks).toHaveLength(1);
    expect($tracks[0]).toEqual(mockTrack);
  });

  it('should add multiple tracks', () => {
    const mockTrack1: Track = {
      id: 'track-1',
      name: 'Main Track',
      type: 'main',
      order: 0,
      clips: [],
      visible: true,
      locked: false,
      volume: 1.0,
    };

    const mockTrack2: Track = {
      id: 'track-2',
      name: 'Overlay Track',
      type: 'overlay',
      order: 1,
      clips: [],
      visible: true,
      locked: false,
      volume: 1.0,
    };

    timelineStore.update((state) => ({
      ...state,
      tracks: [mockTrack1, mockTrack2],
    }));

    const $tracks = get(tracks);
    expect($tracks).toHaveLength(2);
    expect($tracks[0].id).toBe('track-1');
    expect($tracks[1].id).toBe('track-2');
  });

  it('should add a clip to a specific track', () => {
    const mockTrack: Track = {
      id: 'track-1',
      name: 'Main Track',
      type: 'main',
      order: 0,
      clips: [],
      visible: true,
      locked: false,
      volume: 1.0,
    };

    const mockClip: TimelineClip = {
      id: 'timeline-clip-1',
      media_clip_id: 'clip-1',
      track_id: 'track-1',
      start_time: 0,
      in_point: 0,
      out_point: 10.5,
      layer_order: 0,
      transform: null,
    };

    // Add track first
    timelineStore.update((state) => ({
      ...state,
      tracks: [mockTrack],
    }));

    // Add clip to track
    timelineStore.update((state) => ({
      ...state,
      tracks: state.tracks.map((track) => {
        if (track.id === 'track-1') {
          return {
            ...track,
            clips: [...track.clips, mockClip],
          };
        }
        return track;
      }),
    }));

    const $tracks = get(tracks);
    expect($tracks[0].clips).toHaveLength(1);
    expect($tracks[0].clips[0]).toEqual(mockClip);
  });

  it('should remove a clip from timeline', () => {
    const mockTrack: Track = {
      id: 'track-1',
      name: 'Main Track',
      type: 'main',
      order: 0,
      clips: [],
      visible: true,
      locked: false,
      volume: 1.0,
    };

    const mockClip: TimelineClip = {
      id: 'timeline-clip-1',
      media_clip_id: 'clip-1',
      track_id: 'track-1',
      start_time: 0,
      in_point: 0,
      out_point: 10.5,
      layer_order: 0,
      transform: null,
    };

    // Add track and clip
    timelineStore.update((state) => ({
      ...state,
      tracks: [{
        ...mockTrack,
        clips: [mockClip],
      }],
    }));

    expect(get(tracks)[0].clips).toHaveLength(1);

    // Remove clip
    timelineStore.update((state) => ({
      ...state,
      tracks: state.tracks.map((track) => ({
        ...track,
        clips: track.clips.filter((c) => c.id !== 'timeline-clip-1'),
      })),
    }));

    expect(get(tracks)[0].clips).toHaveLength(0);
  });

  it('should calculate timeline duration correctly', () => {
    const mockTrack: Track = {
      id: 'track-1',
      name: 'Main Track',
      type: 'main',
      order: 0,
      clips: [],
      visible: true,
      locked: false,
      volume: 1.0,
    };

    const mockClip1: TimelineClip = {
      id: 'timeline-clip-1',
      media_clip_id: 'clip-1',
      track_id: 'track-1',
      start_time: 0,
      in_point: 0,
      out_point: 10.5,
      layer_order: 0,
      transform: null,
    };

    const mockClip2: TimelineClip = {
      id: 'timeline-clip-2',
      media_clip_id: 'clip-2',
      track_id: 'track-1',
      start_time: 10.5,
      in_point: 0,
      out_point: 15.0,
      layer_order: 0,
      transform: null,
    };

    timelineStore.update((state) => ({
      ...state,
      tracks: [{
        ...mockTrack,
        clips: [mockClip1, mockClip2],
      }],
    }));

    const duration = get(timelineDuration);
    // clip1: 0 to 10.5, clip2: 10.5 to 25.5 (10.5 + (15.0 - 0))
    expect(duration).toBe(25.5);
  });

  it('should return 0 duration for empty timeline', () => {
    const duration = get(timelineDuration);
    expect(duration).toBe(0);
  });

  it('should play the timeline', () => {
    timelineStore.play();
    expect(get(isPlaying)).toBe(true);
  });

  it('should pause the timeline', () => {
    timelineStore.play();
    expect(get(isPlaying)).toBe(true);

    timelineStore.pause();
    expect(get(isPlaying)).toBe(false);
  });

  it('should seek to a specific position', () => {
    timelineStore.seek(15.5);
    expect(get(currentTime)).toBe(15.5);
  });

  it('should update playhead position multiple times', () => {
    timelineStore.seek(10);
    expect(get(currentTime)).toBe(10);

    timelineStore.seek(20);
    expect(get(currentTime)).toBe(20);

    timelineStore.seek(5);
    expect(get(currentTime)).toBe(5);
  });
});
