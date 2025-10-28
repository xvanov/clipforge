// Service layer for Tauri API calls
import { invoke } from '@tauri-apps/api/tauri';
import type { MediaClip } from '$lib/types/clip';
import type { Project } from '$lib/types/project';
import type { TimelineClip, Track } from '$lib/types/timeline';

// Media commands
export async function importMediaFiles(paths: string[]): Promise<MediaClip[]> {
  try {
    const result = await invoke<string>('import_media_files', { paths });
    // TODO: Parse result and return MediaClip[]
    console.log('Import result:', result);
    return [];
  } catch (error) {
    console.error('Failed to import media:', error);
    throw error;
  }
}

export async function getMediaMetadata(clipId: string): Promise<MediaClip> {
  const result = await invoke<string>('get_media_metadata', { clipId });
  // TODO: Parse result
  return JSON.parse(result);
}

// Project commands
export async function createNewProject(name: string): Promise<Project> {
  const result = await invoke<string>('create_new_project', { name });
  return JSON.parse(result);
}

export async function saveProject(path: string): Promise<void> {
  await invoke('save_project', { path });
}

export async function loadProject(path: string): Promise<Project> {
  const result = await invoke<string>('load_project', { path });
  return JSON.parse(result);
}

// Timeline commands
export async function addClipToTimeline(
  mediaClipId: string,
  trackId: string,
  startTime: number
): Promise<TimelineClip> {
  const result = await invoke<string>('add_clip_to_timeline', {
    mediaClipId,
    trackId,
    startTime,
  });
  return JSON.parse(result);
}

export async function updateTimelineClip(clipId: string): Promise<TimelineClip> {
  const result = await invoke<string>('update_timeline_clip', { clipId });
  return JSON.parse(result);
}

export async function splitTimelineClip(clipId: string, splitTime: number): Promise<void> {
  await invoke('split_timeline_clip', { clipId, splitTime });
}

export async function deleteTimelineClip(clipId: string): Promise<void> {
  await invoke('delete_timeline_clip', { clipId });
}

export async function createTrack(name: string, trackType: string): Promise<Track> {
  const result = await invoke<string>('create_track', { name, trackType });
  return JSON.parse(result);
}

