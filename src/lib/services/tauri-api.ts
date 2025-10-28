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
  try {
    return await tauriInvoke('import_media_files', { paths });
  } catch (error) {
    console.error('Failed to import media files:', error);
    throw error;
  }
}

export async function getMediaMetadata(clipId: string): Promise<MediaClip> {
  try {
    return await tauriInvoke('get_media_metadata', { clipId });
  } catch (error) {
    console.error('Failed to get media metadata:', error);
    throw error;
  }
}

export async function generateThumbnailForClip(clipId: string, timestamp: number): Promise<string> {
  try {
    return await tauriInvoke('generate_thumbnail_for_clip', { clipId, timestamp });
  } catch (error) {
    console.error('Failed to generate thumbnail:', error);
    throw error;
  }
}

// Playback Commands
export async function loadClipForPlayback(
  clipId: string,
  useProxy: boolean = false
): Promise<string> {
  try {
    return await tauriInvoke('load_clip_for_playback', { clipId, useProxy });
  } catch (error) {
    console.error('Failed to load clip for playback:', error);
    throw error;
  }
}

// Project Commands
export async function createNewProject(name: string): Promise<Project> {
  try {
    return await tauriInvoke('create_new_project', { name });
  } catch (error) {
    console.error('Failed to create new project:', error);
    throw error;
  }
}

export async function saveProject(path: string): Promise<{ success: boolean; path: string }> {
  try {
    return await tauriInvoke('save_project', { path });
  } catch (error) {
    console.error('Failed to save project:', error);
    throw error;
  }
}

export async function loadProject(path: string): Promise<Project> {
  try {
    return await tauriInvoke('load_project', { path });
  } catch (error) {
    console.error('Failed to load project:', error);
    throw error;
  }
}

// Timeline Commands
export async function addClipToTimeline(
  mediaClipId: string,
  trackId: string,
  startTime: number,
  inPoint: number,
  outPoint: number
): Promise<TimelineClip> {
  try {
    return await tauriInvoke('add_clip_to_timeline', {
      mediaClipId,
      trackId,
      startTime,
      inPoint,
      outPoint,
    });
  } catch (error) {
    console.error('Failed to add clip to timeline:', error);
    throw error;
  }
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
  try {
    return await tauriInvoke('update_timeline_clip', { clipId, updates });
  } catch (error) {
    console.error('Failed to update timeline clip:', error);
    throw error;
  }
}

export async function splitTimelineClip(
  clipId: string,
  splitTime: number
): Promise<{ clipBefore: TimelineClip; clipAfter: TimelineClip }> {
  try {
    return await tauriInvoke('split_timeline_clip', { clipId, splitTime });
  } catch (error) {
    console.error('Failed to split timeline clip:', error);
    throw error;
  }
}

export async function deleteTimelineClip(clipId: string): Promise<void> {
  try {
    return await tauriInvoke('delete_timeline_clip', { clipId });
  } catch (error) {
    console.error('Failed to delete timeline clip:', error);
    throw error;
  }
}

export async function createTrack(name: string, type: 'main' | 'overlay'): Promise<Track> {
  try {
    return await tauriInvoke('create_track', { name, trackType: type });
  } catch (error) {
    console.error('Failed to create track:', error);
    throw error;
  }
}
