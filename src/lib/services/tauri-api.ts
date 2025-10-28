// Tauri API wrapper - provides type-safe wrappers for Tauri commands
import { invoke as tauriInvoke } from '@tauri-apps/api';
import type { MediaClip } from '$lib/types/clip';
import type { Project } from '$lib/types/project';
import type { TimelineClip, Track } from '$lib/types/timeline';

// Re-export invoke for general use
export { invoke } from '@tauri-apps/api';

// Media Commands
export async function importMediaFiles(paths: string[]): Promise<{
  clips: MediaClip[];
  errors: Array<{ path: string; error: string }>;
}> {
  return tauriInvoke('import_media_files', { paths });
}

export async function getMediaMetadata(clipId: string): Promise<MediaClip> {
  return tauriInvoke('get_media_metadata', { clipId });
}

export async function generateThumbnailForClip(
  clipId: string,
  timestamp: number
): Promise<string> {
  return tauriInvoke('generate_thumbnail_for_clip', { clipId, timestamp });
}

// Playback Commands
export async function loadClipForPlayback(
  clipId: string,
  useProxy: boolean = false
): Promise<string> {
  return tauriInvoke('load_clip_for_playback', { clipId, useProxy });
}

// Project Commands
export async function createNewProject(name: string): Promise<Project> {
  return tauriInvoke('create_new_project', { name });
}

export async function saveProject(path: string): Promise<{ success: boolean; path: string }> {
  return tauriInvoke('save_project', { path });
}

export async function loadProject(path: string): Promise<Project> {
  return tauriInvoke('load_project', { path });
}

// Timeline Commands
export async function addClipToTimeline(
  mediaClipId: string,
  trackId: string,
  startTime: number,
  inPoint: number,
  outPoint: number
): Promise<TimelineClip> {
  return tauriInvoke('add_clip_to_timeline', {
    mediaClipId,
    trackId,
    startTime,
    inPoint,
    outPoint,
  });
}

export async function updateTimelineClip(
  clipId: string,
  updates: {
    startTime?: number;
    inPoint?: number;
    outPoint?: number;
    trackId?: string;
  }
): Promise<TimelineClip> {
  return tauriInvoke('update_timeline_clip', { clipId, updates });
}

export async function splitTimelineClip(
  clipId: string,
  splitTime: number
): Promise<{ clipBefore: TimelineClip; clipAfter: TimelineClip }> {
  return tauriInvoke('split_timeline_clip', { clipId, splitTime });
}

export async function deleteTimelineClip(clipId: string): Promise<void> {
  return tauriInvoke('delete_timeline_clip', { clipId });
}

export async function createTrack(
  name: string,
  type: 'main' | 'overlay'
): Promise<Track> {
  return tauriInvoke('create_track', { name, trackType: type });
}
