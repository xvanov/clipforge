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
      tracks: [
        {
          ...mockTrack,
          clips: [mockClip],
        },
      ],
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
      tracks: [
        {
          ...mockTrack,
          clips: [mockClip1, mockClip2],
        },
      ],
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

  // ============================================================================
  // PHASE 4 TESTS: Multiple Clips on Same Track
  // ============================================================================

  describe('Multiple Clips on Same Track', () => {
    it('should add multiple clips to same track sequentially', () => {
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
        media_clip_id: 'media-1',
        track_id: 'track-1',
        start_time: 0,
        in_point: 0,
        out_point: 10.0,
        layer_order: 0,
        transform: null,
      };

      const mockClip2: TimelineClip = {
        id: 'timeline-clip-2',
        media_clip_id: 'media-2',
        track_id: 'track-1',
        start_time: 10.5,
        in_point: 0,
        out_point: 15.0,
        layer_order: 0,
        transform: null,
      };

      const mockClip3: TimelineClip = {
        id: 'timeline-clip-3',
        media_clip_id: 'media-3',
        track_id: 'track-1',
        start_time: 26.0,
        in_point: 0,
        out_point: 8.5,
        layer_order: 0,
        transform: null,
      };

      // Set up initial track
      timelineStore.update((state) => ({
        ...state,
        tracks: [mockTrack],
      }));

      // Add first clip
      timelineStore.update((state) => ({
        ...state,
        tracks: state.tracks.map((track) => {
          if (track.id === 'track-1') {
            return {
              ...track,
              clips: [...track.clips, mockClip1],
            };
          }
          return track;
        }),
      }));

      let $tracks = get(tracks);
      expect($tracks[0].clips).toHaveLength(1);
      expect($tracks[0].clips[0].id).toBe('timeline-clip-1');

      // Add second clip
      timelineStore.update((state) => ({
        ...state,
        tracks: state.tracks.map((track) => {
          if (track.id === 'track-1') {
            return {
              ...track,
              clips: [...track.clips, mockClip2],
            };
          }
          return track;
        }),
      }));

      $tracks = get(tracks);
      expect($tracks[0].clips).toHaveLength(2);
      expect($tracks[0].clips[0].id).toBe('timeline-clip-1');
      expect($tracks[0].clips[1].id).toBe('timeline-clip-2');

      // Add third clip
      timelineStore.update((state) => ({
        ...state,
        tracks: state.tracks.map((track) => {
          if (track.id === 'track-1') {
            return {
              ...track,
              clips: [...track.clips, mockClip3],
            };
          }
          return track;
        }),
      }));

      $tracks = get(tracks);
      expect($tracks[0].clips).toHaveLength(3);
      expect($tracks[0].clips[0].id).toBe('timeline-clip-1');
      expect($tracks[0].clips[1].id).toBe('timeline-clip-2');
      expect($tracks[0].clips[2].id).toBe('timeline-clip-3');
    });

    it('should maintain correct positioning for multiple clips', () => {
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
        media_clip_id: 'media-1',
        track_id: 'track-1',
        start_time: 0,
        in_point: 0,
        out_point: 10.0,
        layer_order: 0,
        transform: null,
      };

      const mockClip2: TimelineClip = {
        id: 'timeline-clip-2',
        media_clip_id: 'media-2',
        track_id: 'track-1',
        start_time: 10.5,
        in_point: 0,
        out_point: 15.0,
        layer_order: 0,
        transform: null,
      };

      const mockClip3: TimelineClip = {
        id: 'timeline-clip-3',
        media_clip_id: 'media-3',
        track_id: 'track-1',
        start_time: 26.0,
        in_point: 0,
        out_point: 8.5,
        layer_order: 0,
        transform: null,
      };

      // Add track with all three clips
      timelineStore.update((state) => ({
        ...state,
        tracks: [
          {
            ...mockTrack,
            clips: [mockClip1, mockClip2, mockClip3],
          },
        ],
      }));

      const $tracks = get(tracks);

      // Verify each clip maintains its position
      expect($tracks[0].clips[0].start_time).toBe(0);
      expect($tracks[0].clips[1].start_time).toBe(10.5);
      expect($tracks[0].clips[2].start_time).toBe(26.0);

      // Verify clips don't overlap (clip1 ends at 10, clip2 starts at 10.5)
      const clip1End =
        $tracks[0].clips[0].start_time +
        ($tracks[0].clips[0].out_point - $tracks[0].clips[0].in_point);
      expect(clip1End).toBe(10.0);
      expect($tracks[0].clips[1].start_time).toBeGreaterThan(clip1End);

      // Verify clip2 ends at 25.5 (10.5 + 15.0)
      const clip2End =
        $tracks[0].clips[1].start_time +
        ($tracks[0].clips[1].out_point - $tracks[0].clips[1].in_point);
      expect(clip2End).toBe(25.5);
      expect($tracks[0].clips[2].start_time).toBeGreaterThan(clip2End);
    });

    it('should calculate timeline duration with multiple clips', () => {
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
        media_clip_id: 'media-1',
        track_id: 'track-1',
        start_time: 0,
        in_point: 0,
        out_point: 10.0,
        layer_order: 0,
        transform: null,
      };

      const mockClip2: TimelineClip = {
        id: 'timeline-clip-2',
        media_clip_id: 'media-2',
        track_id: 'track-1',
        start_time: 10.5,
        in_point: 0,
        out_point: 15.0,
        layer_order: 0,
        transform: null,
      };

      const mockClip3: TimelineClip = {
        id: 'timeline-clip-3',
        media_clip_id: 'media-3',
        track_id: 'track-1',
        start_time: 26.0,
        in_point: 0,
        out_point: 8.5,
        layer_order: 0,
        transform: null,
      };

      timelineStore.update((state) => ({
        ...state,
        tracks: [
          {
            ...mockTrack,
            clips: [mockClip1, mockClip2, mockClip3],
          },
        ],
      }));

      const duration = get(timelineDuration);
      // clip3 ends last: 26.0 + (8.5 - 0) = 34.5
      expect(duration).toBe(34.5);
    });

    it('should persist all clips after multiple additions', () => {
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

      // Initialize with track
      timelineStore.update((state) => ({
        ...state,
        tracks: [mockTrack],
      }));

      // Add 5 clips sequentially
      for (let i = 1; i <= 5; i++) {
        const clip: TimelineClip = {
          id: `timeline-clip-${i}`,
          media_clip_id: `media-${i}`,
          track_id: 'track-1',
          start_time: (i - 1) * 10,
          in_point: 0,
          out_point: 9.0,
          layer_order: 0,
          transform: null,
        };

        timelineStore.update((state) => ({
          ...state,
          tracks: state.tracks.map((track) => {
            if (track.id === 'track-1') {
              return {
                ...track,
                clips: [...track.clips, clip],
              };
            }
            return track;
          }),
        }));
      }

      const $tracks = get(tracks);
      expect($tracks[0].clips).toHaveLength(5);

      // Verify all clips are present with correct IDs
      for (let i = 0; i < 5; i++) {
        expect($tracks[0].clips[i].id).toBe(`timeline-clip-${i + 1}`);
        expect($tracks[0].clips[i].media_clip_id).toBe(`media-${i + 1}`);
      }

      // Verify timeline duration includes all clips
      const duration = get(timelineDuration);
      // Last clip: start=40, duration=9, end=49
      expect(duration).toBe(49.0);
    });
  });

  // ============================================================================
  // INTEGRATION TESTS: Tauri Command Layer
  // ============================================================================

  describe('Tauri Command Integration', () => {
    beforeEach(() => {
      // Reset store
      timelineStore.set({
        tracks: [],
        currentTime: 0,
        isPlaying: false,
        zoom: 50,
      });

      // Clear mock calls
      vi.clearAllMocks();
    });

    it('should handle multiple sequential addClipToTimeline calls', async () => {
      const { invoke } = await import('@tauri-apps/api/tauri');
      const mockInvoke = vi.mocked(invoke);

      // Setup mock track
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
        tracks: [mockTrack],
      }));

      // Mock Tauri responses for 3 different clips
      mockInvoke
        .mockResolvedValueOnce({
          id: 'timeline-clip-1',
          media_clip_id: 'media-1',
          track_id: 'track-1',
          start_time: 0,
          in_point: 0,
          out_point: 10.0,
          layer_order: 0,
          transform: null,
        })
        .mockResolvedValueOnce({
          id: 'timeline-clip-2',
          media_clip_id: 'media-2',
          track_id: 'track-1',
          start_time: 10.5,
          in_point: 0,
          out_point: 15.0,
          layer_order: 0,
          transform: null,
        })
        .mockResolvedValueOnce({
          id: 'timeline-clip-3',
          media_clip_id: 'media-3',
          track_id: 'track-1',
          start_time: 26.0,
          in_point: 0,
          out_point: 8.5,
          layer_order: 0,
          transform: null,
        });

      // Add first clip
      await timelineStore.addClipToTimeline('media-1', 'track-1', 0, 0, 10.0);
      let $tracks = get(tracks);
      expect($tracks[0].clips).toHaveLength(1);

      // Add second clip
      await timelineStore.addClipToTimeline('media-2', 'track-1', 10.5, 0, 15.0);
      $tracks = get(tracks);
      expect($tracks[0].clips).toHaveLength(2);

      // Add third clip
      await timelineStore.addClipToTimeline('media-3', 'track-1', 26.0, 0, 8.5);
      $tracks = get(tracks);
      expect($tracks[0].clips).toHaveLength(3);

      // Verify all three clips are in the track
      expect($tracks[0].clips[0].id).toBe('timeline-clip-1');
      expect($tracks[0].clips[1].id).toBe('timeline-clip-2');
      expect($tracks[0].clips[2].id).toBe('timeline-clip-3');

      // Verify Tauri command was called 3 times
      expect(mockInvoke).toHaveBeenCalledTimes(3);
      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'add_clip_to_timeline', {
        mediaClipId: 'media-1',
        trackId: 'track-1',
        startTime: 0,
        inPoint: 0,
        outPoint: 10.0,
      });
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'add_clip_to_timeline', {
        mediaClipId: 'media-2',
        trackId: 'track-1',
        startTime: 10.5,
        inPoint: 0,
        outPoint: 15.0,
      });
      expect(mockInvoke).toHaveBeenNthCalledWith(3, 'add_clip_to_timeline', {
        mediaClipId: 'media-3',
        trackId: 'track-1',
        startTime: 26.0,
        inPoint: 0,
        outPoint: 8.5,
      });
    });

    it('should maintain state consistency across multiple Tauri calls', async () => {
      const { invoke } = await import('@tauri-apps/api/tauri');
      const mockInvoke = vi.mocked(invoke);

      // Setup mock track
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
        tracks: [mockTrack],
      }));

      // Mock responses
      mockInvoke
        .mockResolvedValueOnce({
          id: 'clip-1',
          media_clip_id: 'media-1',
          track_id: 'track-1',
          start_time: 0,
          in_point: 0,
          out_point: 5.0,
          layer_order: 0,
          transform: null,
        })
        .mockResolvedValueOnce({
          id: 'clip-2',
          media_clip_id: 'media-2',
          track_id: 'track-1',
          start_time: 5.5,
          in_point: 0,
          out_point: 7.0,
          layer_order: 0,
          transform: null,
        });

      // Add clips and check state after each
      const clip1 = await timelineStore.addClipToTimeline('media-1', 'track-1', 0, 0, 5.0);
      expect(clip1.id).toBe('clip-1');
      expect(get(tracks)[0].clips).toHaveLength(1);

      const clip2 = await timelineStore.addClipToTimeline('media-2', 'track-1', 5.5, 0, 7.0);
      expect(clip2.id).toBe('clip-2');
      expect(get(tracks)[0].clips).toHaveLength(2);

      // Verify both clips are still in state (no overwrites)
      const $tracks = get(tracks);
      expect($tracks[0].clips[0].id).toBe('clip-1');
      expect($tracks[0].clips[1].id).toBe('clip-2');
    });

    it('should handle errors without corrupting state', async () => {
      const { invoke } = await import('@tauri-apps/api/tauri');
      const mockInvoke = vi.mocked(invoke);

      // Setup mock track with one clip
      const mockTrack: Track = {
        id: 'track-1',
        name: 'Main Track',
        type: 'main',
        order: 0,
        clips: [
          {
            id: 'existing-clip',
            media_clip_id: 'media-existing',
            track_id: 'track-1',
            start_time: 0,
            in_point: 0,
            out_point: 5.0,
            layer_order: 0,
            transform: null,
          },
        ],
        visible: true,
        locked: false,
        volume: 1.0,
      };

      timelineStore.update((state) => ({
        ...state,
        tracks: [mockTrack],
      }));

      // Mock Tauri command to fail
      mockInvoke.mockRejectedValueOnce(new Error('Track not found'));

      // Try to add clip (should fail)
      await expect(
        timelineStore.addClipToTimeline('media-1', 'invalid-track', 0, 0, 10.0)
      ).rejects.toThrow();

      // Verify existing clip is still there (state not corrupted)
      const $tracks = get(tracks);
      expect($tracks[0].clips).toHaveLength(1);
      expect($tracks[0].clips[0].id).toBe('existing-clip');
    });
  });
});
