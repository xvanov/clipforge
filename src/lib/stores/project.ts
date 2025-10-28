import { writable, type Writable } from 'svelte/store';
import type { Project } from '$lib/types/project';

// Project store - holds current project state
export const project: Writable<Project | null> = writable(null);

// Helper functions for project operations
export function createProject(name: string) {
  // TODO: Call Tauri command to create new project
  console.log('Creating project:', name);
}

export function saveProject(path: string) {
  // TODO: Call Tauri command to save project
  console.log('Saving project to:', path);
}

export function loadProject(path: string) {
  // TODO: Call Tauri command to load project
  console.log('Loading project from:', path);
}

export function markProjectModified() {
  project.update((p) => {
    if (p) {
      return {
        ...p,
        modified_at: new Date().toISOString(),
      };
    }
    return p;
  });
}

