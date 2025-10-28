import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { project, markProjectModified } from './project';
import type { Project } from '$lib/types/project';

describe('Project Store', () => {
  beforeEach(() => {
    // Reset the store before each test
    project.set(null);
  });

  it('should start with null project', () => {
    const currentProject = get(project);
    expect(currentProject).toBeNull();
  });

  it('should set a project', () => {
    const mockProject: Project = {
      id: 'project-1',
      name: 'My Video Project',
      created_at: new Date().toISOString(),
      modified_at: new Date().toISOString(),
      file_path: '/path/to/project.cfp',
      version: '1.0.0',
      tracks: [],
      media_library: [],
      export_settings: {
        resolution: '1080p',
        codec: 'h264',
        quality: 'high',
        fps: 30,
        audio_codec: 'aac',
        audio_bitrate: 128,
        hardware_acceleration: true,
      },
      auto_save_enabled: true,
      last_auto_save: null,
    };

    project.set(mockProject);
    const currentProject = get(project);

    expect(currentProject).not.toBeNull();
    expect(currentProject?.id).toBe('project-1');
    expect(currentProject?.name).toBe('My Video Project');
  });

  it('should mark project as modified by updating modified_at timestamp', () => {
    const now = new Date();
    const earlierTime = new Date(now.getTime() - 10000).toISOString(); // 10 seconds ago

    const mockProject: Project = {
      id: 'project-1',
      name: 'My Video Project',
      created_at: earlierTime,
      modified_at: earlierTime,
      file_path: '/path/to/project.cfp',
      version: '1.0.0',
      tracks: [],
      media_library: [],
      export_settings: {
        resolution: '1080p',
        codec: 'h264',
        quality: 'high',
        fps: 30,
        audio_codec: 'aac',
        audio_bitrate: 128,
        hardware_acceleration: true,
      },
      auto_save_enabled: true,
      last_auto_save: null,
    };

    project.set(mockProject);

    // Wait a tiny bit to ensure timestamp is different
    setTimeout(() => {
      markProjectModified();

      const updatedProject = get(project);
      expect(updatedProject).not.toBeNull();
      expect(updatedProject?.modified_at).not.toBe(earlierTime);

      // Parse and compare dates
      const modifiedDate = new Date(updatedProject!.modified_at);
      const earlierDate = new Date(earlierTime);
      expect(modifiedDate.getTime()).toBeGreaterThan(earlierDate.getTime());
    }, 10);
  });

  it('should not crash when marking non-existent project as modified', () => {
    expect(() => markProjectModified()).not.toThrow();
    expect(get(project)).toBeNull();
  });

  it('should update project state', () => {
    const mockProject: Project = {
      id: 'project-1',
      name: 'Original Name',
      created_at: new Date().toISOString(),
      modified_at: new Date().toISOString(),
      file_path: '/path/to/project.cfp',
      version: '1.0.0',
      tracks: [],
      media_library: [],
      export_settings: {
        resolution: '1080p',
        codec: 'h264',
        quality: 'high',
        fps: 30,
        audio_codec: 'aac',
        audio_bitrate: 128,
        hardware_acceleration: true,
      },
      auto_save_enabled: true,
      last_auto_save: null,
    };

    project.set(mockProject);

    // Update the project name
    project.update((p) => {
      if (p) {
        return { ...p, name: 'Updated Name' };
      }
      return p;
    });

    const updatedProject = get(project);
    expect(updatedProject?.name).toBe('Updated Name');
    expect(updatedProject?.id).toBe('project-1'); // Other properties unchanged
  });
});
