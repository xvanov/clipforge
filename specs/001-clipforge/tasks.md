# Tasks: ClipForge Desktop Video Editor

**Input**: Design documents from `/specs/001-clipforge/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests follow TDD for critical paths (media import, FFmpeg operations, timeline state, export pipeline). Trivial operations may skip tests per constitution guidance. Focus on meaningful test coverage that catches production issues.

**Branching Strategy**: Create a new branch for each phase to isolate work and enable clean PRs.

```bash
# Example: Starting Phase 4 (User Story 2)
git checkout main
git pull origin main
git checkout -b 004-timeline-editing

# After completing phase, push and create PR
git push origin 004-timeline-editing
# Create PR: 004-timeline-editing → main
```

**CI/CD Validation**: After completing each phase, run all CI/CD checks locally to ensure everything passes before moving to the next phase. See README.md for complete instructions.

```bash
# Quick CI/CD validation (run from project root)
npm test && npm run lint && npm run type-check && npm run format:check && \
cd src-tauri && cargo test && cargo clippy -- -D warnings && cargo fmt --check
```

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Desktop app (Tauri)**: `src-tauri/src/` for Rust backend, `src/` for Svelte frontend
- Paths shown below follow Tauri project structure from plan.md

---

## Phase 1: Setup (Shared Infrastructure) ✅ COMPLETE

**Purpose**: Project initialization and basic structure

- [x] T001 Initialize Tauri project with Svelte template (manual setup due to npm permissions)
- [x] T002 [P] Configure Cargo.toml dependencies in src-tauri/Cargo.toml (tauri, serde, tokio, rusqlite)
- [x] T003 [P] Configure package.json dependencies (svelte, vite, typescript, vitest)
- [x] T004 [P] Create .gitignore with Rust and Node.js patterns
- [x] T005 [P] Create GitHub Actions CI/CD workflow in .github/workflows/ci.yml
- [x] T006 [P] Configure tauri.conf.json with app metadata and permissions
- [x] T007 [P] Set up TypeScript configuration in tsconfig.json
- [x] T008 [P] Set up Vite configuration in vite.config.ts
- [x] T009 Create project directory structure per plan.md (commands/, ffmpeg/, models/, storage/, platform/)
- [x] T010 [P] Add test fixtures directory tests/fixtures/ (created structure)

**Checkpoint**: ✅ Phase 1 complete! Run CI/CD checks locally before proceeding to Phase 2.

---

## Phase 2: Foundational (Blocking Prerequisites) ✅ COMPLETE

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T011 Create Rust data models in src-tauri/src/models/mod.rs (module exports)
- [x] T012 [P] Implement Project struct in src-tauri/src/models/project.rs with serde serialization
- [x] T013 [P] Implement MediaClip struct in src-tauri/src/models/clip.rs
- [x] T014 [P] Implement TimelineClip struct in src-tauri/src/models/timeline.rs
- [x] T015 [P] Implement Track struct in src-tauri/src/models/timeline.rs
- [x] T016 [P] Create TypeScript types in src/lib/types/project.ts matching Rust models
- [x] T017 [P] Create TypeScript types in src/lib/types/clip.ts
- [x] T018 [P] Create TypeScript types in src/lib/types/timeline.ts
- [x] T019 Create Svelte stores module (created individual store files)
- [x] T020 [P] Implement project store in src/lib/stores/project.ts using writable store
- [x] T021 [P] Implement media library store in src/lib/stores/media-library.ts
- [x] T022 [P] Implement timeline store in src/lib/stores/timeline.ts
- [x] T023 Create Tauri API service wrapper in src/lib/services/tauri-api.ts with invoke wrappers
- [x] T024 Create SQLite cache database initialization in src-tauri/src/storage/cache.rs
- [x] T025 Implement cache schema (media_clips, auto_saves tables) in src-tauri/src/storage/cache.rs
- [x] T026 Create FFmpeg wrapper module in src-tauri/src/ffmpeg/mod.rs (stub created)

**Checkpoint**: ✅ Foundation complete - Phase 2 done! Run CI/CD checks locally before starting user story implementation.

---

## Phase 3: User Story 1 - Import and Basic Video Playback (Priority: P1) 🎯 MVP

**Goal**: Enable users to import video files and play them back in the app

**Independent Test**: Launch app, import MP4 file via drag-drop, click to play, verify video plays with audio

### Implementation for User Story 1

- [x] T027 [P] [US1] Implement import_media_files Tauri command in src-tauri/src/commands/media.rs
- [x] T028 [P] [US1] Implement get_media_metadata Tauri command in src-tauri/src/commands/media.rs using FFmpeg
- [x] T029 [P] [US1] Create FFmpeg metadata extraction function in src-tauri/src/ffmpeg/metadata.rs
- [x] T030 [P] [US1] Create FFmpeg thumbnail generation function in src-tauri/src/ffmpeg/thumbnails.rs
- [x] T031 [US1] Implement thumbnail generation task queue in src-tauri/src/ffmpeg/thumbnails.rs (async with tokio)
- [x] T032 [P] [US1] Create MediaLibrary Svelte component in src/lib/components/MediaLibrary.svelte
- [x] T033 [P] [US1] Create MediaClipCard Svelte component in src/lib/components/MediaClipCard.svelte (thumbnail + metadata)
- [x] T034 [P] [US1] Create VideoPreview Svelte component in src/lib/components/VideoPreview.svelte with HTML5 video element
- [x] T035 [US1] Implement drag-and-drop file import in MediaLibrary component using Tauri file drop API
- [x] T036 [US1] Implement file picker dialog integration in MediaLibrary component
- [x] T037 [US1] Implement load_clip_for_playback Tauri command in src-tauri/src/commands/playback.rs
- [x] T038 [US1] Wire up VideoPreview playback controls (play, pause, seek) in src/lib/components/VideoPreview.svelte
- [x] T039 [US1] Create main app layout in src/App.svelte with MediaLibrary and VideoPreview sections
- [x] T040 [US1] Implement media library state synchronization between Rust and Svelte stores
- [x] T040b [US1] Implement proxy video generation for MOV and non-web-compatible formats in src-tauri/src/ffmpeg/proxy.rs (background async)

**Checkpoint**: ✅ User Story 1 complete! Users can import and play videos. **Run full CI/CD validation before proceeding to Phase 4.**

---

## Phase 4: User Story 2 - Timeline Editing and Trimming (Priority: P1) 🎯 MVP

**Goal**: Enable users to arrange clips on timeline, trim them, and preview edits

**Independent Test**: Import 2-3 clips, drag onto timeline, trim by setting in/out points, split clip, preview sequence

### Implementation for User Story 2

- [x] T041 [P] [US2] Create Timeline Svelte component in src/lib/components/Timeline.svelte
- [x] T042 [P] [US2] Create TrackView Svelte component in src/lib/components/TrackView.svelte
- [x] T043 [P] [US2] Create TimelineClipView Svelte component in src/lib/components/TimelineClipView.svelte
- [x] T044 [P] [US2] Create timeline Canvas renderer in src/lib/canvas/timeline-renderer.ts
- [x] T045 [P] [US2] Create clip renderer in src/lib/canvas/clip-renderer.ts
- [x] T046 [P] [US2] Create playhead renderer in src/lib/canvas/playhead-renderer.ts
- [x] T047 [US2] Implement canvas rendering loop with requestAnimationFrame in timeline-renderer.ts
- [x] T048 [US2] Implement add_clip_to_timeline Tauri command in src-tauri/src/commands/timeline.rs
- [x] T049 [US2] Implement update_timeline_clip Tauri command in src-tauri/src/commands/timeline.rs
- [x] T050 [US2] Implement split_timeline_clip Tauri command in src-tauri/src/commands/timeline.rs
- [x] T051 [US2] Implement delete_timeline_clip Tauri command in src-tauri/src/commands/timeline.rs
- [x] T052 [US2] Implement create_track Tauri command in src-tauri/src/commands/timeline.rs
- [x] T053 [US2] Implement drag-and-drop from media library to timeline in Timeline.svelte
- [x] T054 [US2] Implement clip reordering via drag in Timeline.svelte
- [x] T055 [US2] Implement trim controls (in/out point markers) in TimelineClipView.svelte
- [x] T056 [US2] Implement split at playhead functionality in Timeline.svelte
- [x] T057 [US2] Implement timeline zoom controls in Timeline.svelte
- [x] T058 [US2] Implement timeline scrubbing (drag playhead) in Timeline.svelte
- [x] T059 [US2] Wire up timeline playback to VideoPreview (switch clips on transition)
- [x] T060 [US2] Implement timeline state validation (prevent overlaps) in src-tauri/src/commands/timeline.rs
- [x] T061 [US2] Update main app layout to include Timeline component below preview

**Checkpoint**: ✅ User Stories 1 AND 2 complete! Users can import, play, and edit videos on timeline. **Run full CI/CD validation before proceeding to Phase 5.**

---

## Phase 5: User Story 3 - Video Export (Priority: P1) 🎯 MVP

**Goal**: Enable users to export edited timeline as MP4 video file

**Independent Test**: Create timeline with 2-3 clips, click Export, select 1080p, wait for completion, play exported file

### Implementation for User Story 3

- [x] T062 [P] [US3] Create ExportDialog Svelte component in src/lib/components/ExportDialog.svelte
- [x] T063 [P] [US3] Create ExportSettings struct in src-tauri/src/models/export.rs
- [x] T064 [P] [US3] Create FFmpeg export pipeline in src-tauri/src/ffmpeg/export.rs
- [x] T065 [US3] Implement FFmpeg concat file generation from timeline in src-tauri/src/ffmpeg/export.rs
- [x] T066 [US3] Implement export_timeline Tauri command in src-tauri/src/commands/export.rs
- [x] T067 [US3] Implement cancel_export Tauri command in src-tauri/src/commands/export.rs
- [x] T068 [US3] Implement FFmpeg progress parsing (stderr) in src-tauri/src/ffmpeg/export.rs
- [x] T069 [US3] Implement Tauri event emission for export progress in src-tauri/src/commands/export.rs
- [x] T070 [US3] Wire up export dialog UI with resolution options in ExportDialog.svelte
- [x] T071 [US3] Implement progress bar in ExportDialog.svelte listening to export_progress events
- [x] T072 [US3] Implement export cancellation UI in ExportDialog.svelte
- [x] T073 [US3] Implement export completion notification in ExportDialog.svelte
- [x] T074 [US3] Add Export button to main app toolbar in src/App.svelte
- [x] T075 [US3] Implement partial file cleanup on cancel/error in src-tauri/src/commands/export.rs

**Checkpoint**: 🎉 **MVP COMPLETE!** Users can import, edit, and export videos (full workflow). **Run full CI/CD validation to confirm MVP stability.**

---

## Phase 6: User Story 4 - Screen and Webcam Recording (Priority: P2)

**Goal**: Enable users to record screen and webcam directly in the app

**Independent Test**: Click Record Screen, capture 30s, stop, verify clip in media library; repeat for webcam

### Implementation for User Story 4

- [x] T076 [P] [US4] Create RecordingSession struct in src-tauri/src/models/recording.rs
- [x] T077 [P] [US4] Create RecordingControls Svelte component in src/lib/components/RecordingControls.svelte
- [x] T078 [P] [US4] Create platform-specific module structure in src-tauri/src/platform/mod.rs
- [x] T079 [US4] Implement request_recording_permissions Tauri command in src-tauri/src/commands/recording.rs
- [x] T080 [US4] Implement list_recording_sources Tauri command in src-tauri/src/commands/recording.rs
- [x] T081 [US4] Implement start_recording Tauri command in src-tauri/src/commands/recording.rs
- [x] T082 [US4] Implement stop_recording Tauri command in src-tauri/src/commands/recording.rs
- [x] T083 [US4] Implement macOS screen recording using AVFoundation in src-tauri/src/platform/macos.rs (conditional compilation)
- [x] T084 [US4] Implement Windows screen recording using Windows.Graphics.Capture in src-tauri/src/platform/windows.rs (conditional compilation)
- [x] T085 [US4] Implement recording duration tracking with Tauri events in src-tauri/src/commands/recording.rs
- [x] T086 [US4] Create screen source selector UI in RecordingControls.svelte
- [x] T087 [US4] Create webcam preview UI in RecordingControls.svelte
- [x] T088 [US4] Implement recording indicator (elapsed time) in RecordingControls.svelte
- [x] T089 [US4] Wire up recording completion to media library (auto-import) in src-tauri/src/commands/recording.rs
- [x] T090 [US4] Add Recording panel to main app layout in src/App.svelte

**Checkpoint**: ✅ User Story 4 complete! Users can record screen/webcam in addition to importing files. **Run full CI/CD validation before proceeding to Phase 6.5.**

---

## Phase 6.5: User Story 6 - AI Speech-to-Text Captions (Priority: P4)

**Goal**: Enable users to automatically generate captions from video audio using local AI processing

**Independent Test**: Import video with speech, click "Generate Captions", verify timestamped captions appear synced to audio

### Implementation for User Story 6

- [x] T091 [P] [US6] Create Caption struct serialization in src-tauri/src/models/caption.rs
- [x] T092 [P] [US6] Create TypeScript Caption types in src/lib/types/caption.ts
- [x] T093 [P] [US6] Implement audio extraction function in src-tauri/src/ffmpeg/audio.rs (WAV PCM format)
- [x] T094 [US6] Create Whisper.cpp integration module in src-tauri/src/ai/whisper.rs
- [x] T095 [US6] Implement Whisper.cpp CLI wrapper for transcription in src-tauri/src/ai/whisper.rs
- [x] T096 [US6] Implement SRT/JSON parser for Whisper output in src-tauri/src/ai/whisper.rs
- [x] T097 [US6] Implement generate_captions Tauri command in src-tauri/src/commands/captions.rs
- [x] T098 [US6] Implement caption progress events in src-tauri/src/commands/captions.rs
- [x] T099 [P] [US6] Create CaptionsPanel Svelte component in src/lib/components/CaptionsPanel.svelte
- [x] T100 [P] [US6] Create CaptionTrack canvas renderer in src/lib/canvas/caption-renderer.ts
- [x] T101 [US6] Implement caption editing UI in CaptionsPanel.svelte (text, timing)
- [x] T102 [US6] Add Generate Captions button to MediaClipCard component
- [x] T103 [US6] Implement caption overlay in VideoPreview component
- [x] T104 [US6] Add captions to export pipeline in src-tauri/src/ffmpeg/export.rs (burn-in or SRT)
- [x] T105 [US6] Wire up caption generation progress events in frontend

**Checkpoint**: ✅ User Story 6 complete! Users can generate AI captions for accessible content. **Run full CI/CD validation before proceeding to Phase 7.**

---

## Phase 7: User Story 5 - Multi-Track Timeline with Overlay Support (Priority: P3)

**Goal**: Enable users to create picture-in-picture effects with multiple video tracks

**Independent Test**: Place clip on track 1, add clip to track 2, resize/position overlay, export to verify composition

### Implementation for User Story 5

- [ ] T091 [P] [US5] Create Transform struct in src-tauri/src/models/timeline.rs (x, y, width, height, rotation)
- [ ] T092 [P] [US5] Add transform field to TimelineClip struct in src-tauri/src/models/timeline.rs
- [ ] T093 [US5] Update canvas clip renderer to support overlay positioning in src/lib/canvas/clip-renderer.ts
- [ ] T094 [US5] Implement multi-track rendering in timeline canvas in src/lib/canvas/timeline-renderer.ts
- [ ] T095 [US5] Implement overlay resize handles in VideoPreview.svelte
- [ ] T096 [US5] Implement overlay drag-to-reposition in VideoPreview.svelte
- [ ] T097 [US5] Update export pipeline to composite multiple tracks in src-tauri/src/ffmpeg/export.rs
- [ ] T098 [US5] Implement FFmpeg overlay filters for multi-track export in src-tauri/src/ffmpeg/export.rs
- [ ] T099 [US5] Add track creation UI in Timeline.svelte
- [ ] T100 [US5] Implement track layer priority (z-index) in Timeline.svelte

**Checkpoint**: ✅ User Story 5 complete! Users can create professional multi-track compositions. **Run full CI/CD validation before proceeding to Phase 8.**

---

## Phase 8: Project Management

**Goal**: Enable users to save and load projects

**Independent Test**: Create project with clips on timeline, save to file, close app, reopen, load project

- [ ] T101 [P] Create project serialization functions in src-tauri/src/storage/project_io.rs
- [ ] T102 [P] Implement save_project Tauri command in src-tauri/src/commands/project.rs
- [ ] T103 [P] Implement load_project Tauri command in src-tauri/src/commands/project.rs
- [ ] T104 [P] Implement create_new_project Tauri command in src-tauri/src/commands/project.rs
- [ ] T105 Implement auto_save_project Tauri command with periodic trigger in src-tauri/src/commands/project.rs
- [ ] T106 Implement auto-save to SQLite cache in src-tauri/src/storage/cache.rs
- [ ] T107 Create Save/Load dialogs using Tauri file dialogs in src/App.svelte
- [ ] T108 Implement unsaved changes detection in project store
- [ ] T109 Implement "Save before close" prompt in src/App.svelte
- [ ] T110 Add File menu with New/Open/Save actions in src/App.svelte

**Checkpoint**: ✅ Phase 8 complete! Projects can be saved and resumed. **Run full CI/CD validation before proceeding to Phase 9.**

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T111 [P] Implement error toast notifications component in src/lib/components/Toast.svelte
- [ ] T112 [P] Add global error handling for Tauri command failures in src/lib/services/tauri-api.ts
- [ ] T113 [P] Implement keyboard shortcuts (play/pause, split, delete) in src/App.svelte
- [ ] T114 [P] Add loading states to long operations (import, export) across components
- [ ] T115 [P] Implement undo/redo state management in timeline store
- [ ] T116 [P] Create app toolbar with common actions in src/lib/components/Toolbar.svelte
- [ ] T117 [P] Add application icon and splash screen in src-tauri/icons/
- [ ] T118 [P] Configure app bundle metadata in src-tauri/tauri.conf.json
- [ ] T119 [P] Add FFmpeg binary bundling in src-tauri/tauri.conf.json resources
- [ ] T120 [P] Create README.md with setup instructions and screenshots
- [ ] T121 Verify CI/CD pipeline runs successfully for macOS build
- [ ] T122 Verify CI/CD pipeline runs successfully for Windows build
- [ ] T123 Test production build on macOS (create .app bundle)
- [ ] T124 Test production build on Windows (create .msi installer)

**Checkpoint**: 🎉 **PRODUCTION READY!** All polish complete, builds verified on all platforms. **Run final CI/CD validation before release.**

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Project Management (Phase 8)**: Can start after Foundational, runs parallel to user stories
- **Polish (Phase 9)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Foundational - Depends on US1 for media library state
- **User Story 3 (P1)**: Can start after Foundational - Depends on US2 for timeline state
- **User Story 4 (P2)**: Can start after Foundational - No dependencies on P1 stories (creates new media clips)
- **User Story 5 (P3)**: Can start after US2 complete - Extends timeline functionality

### Within Each User Story

- Models/types before services and commands
- Rust commands before Svelte components that call them
- Core components before integration/wiring tasks
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Models and types within each story marked [P] can run in parallel
- UI components marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members (after Foundational complete)

---

## Parallel Example: User Story 1

```bash
# Launch all parallel tasks for US1 together:
T027: Implement import_media_files command
T028: Implement get_media_metadata command
T029: Create FFmpeg metadata extraction
T030: Create FFmpeg thumbnail generation
T032: Create MediaLibrary component
T033: Create MediaClipCard component
T034: Create VideoPreview component

# Then run sequential tasks:
T031: Implement thumbnail task queue (depends on T030)
T035: Implement drag-drop import (depends on T027, T032)
# ... and so on
```

---

## Implementation Strategy

### MVP First (User Stories 1-3 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Import & Playback)
4. **STOP and VALIDATE**: Test US1 independently
5. Complete Phase 4: User Story 2 (Timeline Editing)
6. **STOP and VALIDATE**: Test US1 + US2 together
7. Complete Phase 5: User Story 3 (Export)
8. **STOP and VALIDATE**: Test full MVP workflow (import → edit → export)
9. Deploy/demo MVP

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP v0.1!)
3. Add User Story 2 → Test independently → Deploy/Demo (MVP v0.2)
4. Add User Story 3 → Test independently → Deploy/Demo (MVP v1.0 - complete editing workflow)
5. Add User Story 4 → Test independently → Deploy/Demo (v1.1 - recording)
6. Add User Story 5 → Test independently → Deploy/Demo (v1.2 - multi-track)
7. Add Project Management (Phase 8) → Deploy/Demo (v1.3 - save/load)
8. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (T027-T040)
   - Developer B: User Story 2 (T041-T061)
   - Developer C: User Story 3 (T062-T075)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- MVP = User Stories 1-3 only (import → edit → export)
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- Tests are OPTIONAL - focus on implementation first, add tests later if needed

---

## Summary

**Total Tasks**: 139 tasks

**Tasks by Phase**:

- Phase 1 (Setup): 10 tasks
- Phase 2 (Foundational): 16 tasks (CRITICAL - blocks everything)
- Phase 3 (US1 - Import & Playback): 14 tasks
- Phase 4 (US2 - Timeline Editing): 21 tasks
- Phase 5 (US3 - Export): 14 tasks
- Phase 6 (US4 - Recording): 15 tasks
- Phase 6.5 (US6 - AI Captions): 15 tasks
- Phase 7 (US5 - Multi-Track): 10 tasks
- Phase 8 (Project Management): 10 tasks
- Phase 9 (Polish): 14 tasks

**MVP Tasks**: 10 (Setup) + 16 (Foundational) + 14 (US1) + 21 (US2) + 14 (US3) = **75 tasks for complete MVP**

**Parallel Tasks**: 49 tasks marked [P] can run in parallel within their phase

**Estimated Timeline**:

- MVP (P1 stories): 3-4 weeks for experienced developer
- - Recording (P2): +1 week
- - Multi-track (P3): +1 week
- - Project Management: +3-4 days
- - Polish: +1 week
- **Total**: 6-8 weeks for full v1.0

**Ready for**: `/speckit.implement` command to begin implementation
