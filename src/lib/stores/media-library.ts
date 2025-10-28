import { writable, derived, get, type Writable } from 'svelte/store';
import type { MediaClip } from '$lib/types/clip';

// Media library store - holds all imported clips
export const mediaLibrary: Writable<MediaClip[]> = writable([]);

// Derived store for filtered/sorted clips
export const sortedMediaLibrary = derived(mediaLibrary, ($clips) => {
  return [...$clips].sort((a, b) => {
    return new Date(b.imported_at).getTime() - new Date(a.imported_at).getTime();
  });
});

// Helper functions
export function addClipToLibrary(clip: MediaClip) {
  mediaLibrary.update((clips) => [...clips, clip]);
}

export function removeClipFromLibrary(clipId: string) {
  mediaLibrary.update((clips) => clips.filter((c) => c.id !== clipId));
}

export function getClipById(clipId: string): MediaClip | undefined {
  return get(mediaLibrary).find((c) => c.id === clipId);
}
