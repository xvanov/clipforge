import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import {
  mediaLibrary,
  sortedMediaLibrary,
  addClipToLibrary,
  removeClipFromLibrary,
  getClipById,
} from './media-library';
import type { MediaClip } from '$lib/types/clip';

describe('Media Library Store', () => {
  beforeEach(() => {
    // Reset the store before each test
    mediaLibrary.set([]);
  });

  it('should start with an empty media library', () => {
    const clips = get(mediaLibrary);
    expect(clips).toEqual([]);
  });

  it('should add a clip to the library', () => {
    const mockClip: MediaClip = {
      id: 'clip-1',
      name: 'Test Video.mp4',
      source_path: '/path/to/video.mp4',
      proxy_path: null,
      thumbnail_path: null,
      duration: 10.5,
      resolution: '1920x1080',
      width: 1920,
      height: 1080,
      fps: 30,
      codec: 'h264',
      audio_codec: 'aac',
      file_size: 1024000,
      bitrate: 5000,
      has_audio: true,
      imported_at: new Date().toISOString(),
      captions: [],
    };

    addClipToLibrary(mockClip);
    const clips = get(mediaLibrary);

    expect(clips).toHaveLength(1);
    expect(clips[0]).toEqual(mockClip);
  });

  it('should add multiple clips to the library', () => {
    const mockClip1: MediaClip = {
      id: 'clip-1',
      name: 'Video 1.mp4',
      source_path: '/path/to/video1.mp4',
      proxy_path: null,
      thumbnail_path: null,
      duration: 10.5,
      resolution: '1920x1080',
      width: 1920,
      height: 1080,
      fps: 30,
      codec: 'h264',
      audio_codec: 'aac',
      file_size: 1024000,
      bitrate: 5000,
      has_audio: true,
      imported_at: new Date().toISOString(),
      captions: [],
    };

    const mockClip2: MediaClip = {
      id: 'clip-2',
      name: 'Video 2.mp4',
      source_path: '/path/to/video2.mp4',
      proxy_path: null,
      thumbnail_path: null,
      duration: 15.0,
      resolution: '1920x1080',
      width: 1920,
      height: 1080,
      fps: 30,
      codec: 'h264',
      audio_codec: 'aac',
      file_size: 2048000,
      bitrate: 8000,
      has_audio: true,
      imported_at: new Date().toISOString(),
      captions: [],
    };

    addClipToLibrary(mockClip1);
    addClipToLibrary(mockClip2);

    const clips = get(mediaLibrary);
    expect(clips).toHaveLength(2);
    expect(clips[0].id).toBe('clip-1');
    expect(clips[1].id).toBe('clip-2');
  });

  it('should remove a clip from the library by id', () => {
    const mockClip: MediaClip = {
      id: 'clip-1',
      name: 'Test Video.mp4',
      source_path: '/path/to/video.mp4',
      proxy_path: null,
      thumbnail_path: null,
      duration: 10.5,
      resolution: '1920x1080',
      width: 1920,
      height: 1080,
      fps: 30,
      codec: 'h264',
      audio_codec: 'aac',
      file_size: 1024000,
      bitrate: 5000,
      has_audio: true,
      imported_at: new Date().toISOString(),
      captions: [],
    };

    addClipToLibrary(mockClip);
    expect(get(mediaLibrary)).toHaveLength(1);

    removeClipFromLibrary('clip-1');
    expect(get(mediaLibrary)).toHaveLength(0);
  });

  it('should retrieve a clip by id', () => {
    const mockClip: MediaClip = {
      id: 'clip-1',
      name: 'Test Video.mp4',
      source_path: '/path/to/video.mp4',
      proxy_path: null,
      thumbnail_path: null,
      duration: 10.5,
      resolution: '1920x1080',
      width: 1920,
      height: 1080,
      fps: 30,
      codec: 'h264',
      audio_codec: 'aac',
      file_size: 1024000,
      bitrate: 5000,
      has_audio: true,
      imported_at: new Date().toISOString(),
      captions: [],
    };

    addClipToLibrary(mockClip);
    const retrievedClip = getClipById('clip-1');

    expect(retrievedClip).toBeDefined();
    expect(retrievedClip?.id).toBe('clip-1');
    expect(retrievedClip?.name).toBe('Test Video.mp4');
  });

  it('should return undefined for non-existent clip id', () => {
    const retrievedClip = getClipById('non-existent-id');
    expect(retrievedClip).toBeUndefined();
  });

  it('should sort clips by import date (newest first)', () => {
    const now = new Date();
    const mockClip1: MediaClip = {
      id: 'clip-1',
      name: 'Old Video.mp4',
      source_path: '/path/to/old.mp4',
      proxy_path: null,
      thumbnail_path: null,
      duration: 10.5,
      resolution: '1920x1080',
      width: 1920,
      height: 1080,
      fps: 30,
      codec: 'h264',
      audio_codec: 'aac',
      file_size: 1024000,
      bitrate: 5000,
      has_audio: true,
      imported_at: new Date(now.getTime() - 1000000).toISOString(),
      captions: [],
    };

    const mockClip2: MediaClip = {
      id: 'clip-2',
      name: 'New Video.mp4',
      source_path: '/path/to/new.mp4',
      proxy_path: null,
      thumbnail_path: null,
      duration: 15.0,
      resolution: '1920x1080',
      width: 1920,
      height: 1080,
      fps: 30,
      codec: 'h264',
      audio_codec: 'aac',
      file_size: 2048000,
      bitrate: 8000,
      has_audio: true,
      imported_at: now.toISOString(),
      captions: [],
    };

    addClipToLibrary(mockClip1);
    addClipToLibrary(mockClip2);

    const sortedClips = get(sortedMediaLibrary);
    expect(sortedClips).toHaveLength(2);
    expect(sortedClips[0].id).toBe('clip-2'); // Newest first
    expect(sortedClips[1].id).toBe('clip-1');
  });
});
