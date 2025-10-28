import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import {
  timeline,
  playheadPosition,
  isPlaying,
  timelineZoom,
  timelineDuration,
  addTrack,
  addClipToTrack,
  removeClipFromTimeline,
  play,
  pause,
  seek,
} from './timeline';
import type { Track, TimelineClip } from '$lib/types/timeline';

describe('Timeline Store', () => {
  beforeEach(() => {
    // Reset the stores before each test
    timeline.set([]);
    playheadPosition.set(0);
    isPlaying.set(false);
    timelineZoom.set(50);
  });

  it('should start with empty timeline', () => {
    const tracks = get(timeline);
    expect(tracks).toEqual([]);
  });

  it('should start with playhead at 0', () => {
    const position = get(playheadPosition);
    expect(position).toBe(0);
  });

  it('should start in paused state', () => {
    const playing = get(isPlaying);
    expect(playing).toBe(false);
  });

  it('should start with default zoom level', () => {
    const zoom = get(timelineZoom);
    expect(zoom).toBe(50);
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

    addTrack(mockTrack);
    const tracks = get(timeline);

    expect(tracks).toHaveLength(1);
    expect(tracks[0]).toEqual(mockTrack);
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

    addTrack(mockTrack1);
    addTrack(mockTrack2);

    const tracks = get(timeline);
    expect(tracks).toHaveLength(2);
    expect(tracks[0].id).toBe('track-1');
    expect(tracks[1].id).toBe('track-2');
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

    addTrack(mockTrack);
    addClipToTrack('track-1', mockClip);

    const tracks = get(timeline);
    expect(tracks[0].clips).toHaveLength(1);
    expect(tracks[0].clips[0]).toEqual(mockClip);
  });

  it('should not add clip to non-existent track', () => {
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

    addClipToTrack('non-existent-track', mockClip);

    const tracks = get(timeline);
    expect(tracks).toHaveLength(0);
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

    addTrack(mockTrack);
    addClipToTrack('track-1', mockClip);

    expect(get(timeline)[0].clips).toHaveLength(1);

    removeClipFromTimeline('timeline-clip-1');

    expect(get(timeline)[0].clips).toHaveLength(0);
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

    addTrack(mockTrack);
    addClipToTrack('track-1', mockClip1);
    addClipToTrack('track-1', mockClip2);

    const duration = get(timelineDuration);
    // clip1: 0 to 10.5, clip2: 10.5 to 25.5 (10.5 + (15.0 - 0))
    expect(duration).toBe(25.5);
  });

  it('should return 0 duration for empty timeline', () => {
    const duration = get(timelineDuration);
    expect(duration).toBe(0);
  });

  it('should play the timeline', () => {
    play();
    expect(get(isPlaying)).toBe(true);
  });

  it('should pause the timeline', () => {
    play();
    expect(get(isPlaying)).toBe(true);

    pause();
    expect(get(isPlaying)).toBe(false);
  });

  it('should seek to a specific position', () => {
    seek(15.5);
    expect(get(playheadPosition)).toBe(15.5);
  });

  it('should update playhead position multiple times', () => {
    seek(10);
    expect(get(playheadPosition)).toBe(10);

    seek(20);
    expect(get(playheadPosition)).toBe(20);

    seek(5);
    expect(get(playheadPosition)).toBe(5);
  });
});
