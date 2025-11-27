# FFNodes Production Readiness TODO

> **Last Updated:** 2025-11-26
> **Total Items:** 52
> **Status:** Not Started

---

## 🚨 User-Requested Features (Priority: CRITICAL)

### Client UI & UX Fixes

- [ ] **Fix CurrentJob.tsx Progress Display Not Updating**
  - **Severity:** CRITICAL
  - **Files:** `src-tauri/src/job_manager.rs:230`, `src/pages/Dashboard.tsx:125-134`, `src-tauri/src/api.rs:22-36`
  - **Issue:** `totalFrames` field never set, causing UI to always show 0 frames
  - **Fix:** Create `JobStartedPayload` struct with total_frames, emit it in job-started event, update Dashboard listener

- [ ] **Remove Card Gradient Variant**
  - **Severity:** HIGH
  - **Files:** `src/components/ui/Card.tsx:5,18`, `src/components/CurrentJob.tsx:30`
  - **Issue:** Unwanted gradient effect on cards
  - **Fix:** Remove 'gradient' from Card variant types, replace all `variant="gradient"` with "glass" or "solid"

- [ ] **Change App Start Behavior - Start Paused by Default**
  - **Severity:** HIGH
  - **Files:** `src/pages/Dashboard.tsx:69-92`, `src-tauri/src/config.rs`, `src-tauri/src/job_manager.rs:56-74`
  - **Issue:** App auto-starts processing jobs without user confirmation
  - **Fix:** Add `auto_start_processing: bool` to config (default false), show "Start Processing" button when paused, add visual state indicator

- [ ] **Remove Old /setup Route and Page**
  - **Severity:** MEDIUM
  - **Files:** `src/main.tsx:40`, `src/pages/Setup.tsx`, `src/pages/Dashboard.tsx:42`
  - **Issue:** Duplicate setup functionality (Login page has full setup flow)
  - **Fix:** Delete Setup.tsx, remove route from main.tsx, update Dashboard navigation to use /login

---

## 🔴 CRITICAL Production-Blocking Issues (Priority: CRITICAL)

### Security Vulnerabilities

- [ ] **Implement JWT Authentication System**
  - **Severity:** CRITICAL - Complete security bypass
  - **Files:** `server/src/api/auth.rs:8-42`, all API endpoints
  - **Issue:** Server only validates server_guid, returns UUID as "auth token", no actual auth
  - **Fix:** Generate signed JWT on handshake, add auth middleware to validate tokens, update all endpoints

- [ ] **Fix Path Traversal Vulnerability in File Downloads**
  - **Severity:** CRITICAL - Arbitrary file read
  - **Files:** `server/src/api/files.rs:14-76`
  - **Issue:** File paths from database used without validation/canonicalization
  - **Fix:** Add path validation helper, canonicalize all paths, verify within allowed directories

### Race Conditions & Data Integrity

- [ ] **Fix TOCTOU Race Condition in Job Assignment**
  - **Severity:** HIGH - Duplicate job execution
  - **Files:** `server/src/jobs/queue.rs:56-77`
  - **Issue:** Non-atomic GET-then-UPDATE allows multiple clients to get same job
  - **Fix:** Use single atomic UPDATE with WHERE status='pending' RETURNING *

- [ ] **Fix TOCTOU Race in Stale Job Detection**
  - **Severity:** HIGH - Job executed twice
  - **Files:** `server/src/jobs/queue.rs:240-253`
  - **Issue:** Job could start between stale check and requeue
  - **Fix:** Add transaction isolation, use SELECT FOR UPDATE

- [ ] **Add Transaction Isolation to complete_job()**
  - **Severity:** HIGH - Data corruption
  - **Files:** `server/src/jobs/queue.rs:128-152`
  - **Issue:** Two UPDATE queries without transaction, crash causes inconsistent state
  - **Fix:** Wrap all multi-query operations in transactions

---

## 🔵 HIGH Priority Issues

### Job State Management (Client)

- [ ] **Implement Job State Persistence to Disk**
  - **Severity:** HIGH - Data loss on crash/restart
  - **Files:** `src-tauri/src/job_manager.rs`, new: `src-tauri/src/job_state.rs`
  - **Issue:** Jobs only in memory, lost on restart
  - **Fix:** Create JobStateStore struct, save to platform config dir + "job_state.json", save on start/progress/shutdown

- [ ] **Implement FFmpeg Seek-Based Resume with -ss**
  - **Severity:** HIGH - Cannot resume interrupted jobs
  - **Files:** `src-tauri/src/encoder.rs`
  - **Issue:** Cannot resume encoding from last frame
  - **Fix:** Add encode_video_resumable(), calculate seek_seconds from last_frame/fps, add -ss before -i, adjust progress calculation

- [ ] **Add Resume Prompt on App Startup**
  - **Severity:** HIGH - UX for interrupted jobs
  - **Files:** `src-tauri/src/lib.rs` (setup)
  - **Issue:** No way to resume interrupted jobs
  - **Fix:** Load job_state.json on startup, show notification with Resume/Cancel/Restart options

- [ ] **Implement Graceful Shutdown Handler**
  - **Severity:** HIGH - Job state loss on close
  - **Files:** `src-tauri/src/lib.rs`
  - **Issue:** Closing window kills FFmpeg immediately, loses progress
  - **Fix:** Add window close handler, save job state, send SIGTERM to FFmpeg, wait 5s before force kill

### Missing Features & Validation

- [ ] **Add File Upload Size Validation**
  - **Severity:** HIGH - DoS via large uploads
  - **Files:** `server/src/api/files.rs:99`
  - **Issue:** No size limits, unbounded uploads
  - **Fix:** Add configurable max size (default 10GB), streaming validation, rate limiting

- [ ] **Implement WebSocket Broadcasting**
  - **Severity:** HIGH - Missing real-time updates
  - **Files:** `server/src/api/websocket.rs:88-92`
  - **Issue:** _broadcast_event() is stub with TODO
  - **Fix:** Implement WebSocketManager with Arc<Mutex<HashMap>>, register clients, broadcast events

- [ ] **Add Server Graceful Shutdown**
  - **Severity:** HIGH - Data loss on server stop
  - **Files:** `server/src/lib.rs`
  - **Issue:** No SIGTERM/SIGINT handler, abrupt shutdown
  - **Fix:** Add signal handlers, stop accepting jobs, mark in-progress as pending, wait for requests, close DB

- [ ] **Add FFmpeg Binary Checksum Validation**
  - **Severity:** HIGH - Security/reliability
  - **Files:** `ffmpeg/src/lib.rs:173-234`
  - **Issue:** Downloaded binaries not validated
  - **Fix:** Fetch checksums from ffbinaries.com, verify SHA256, test with --version

---

## 🟡 MEDIUM Priority Issues

### Configuration & Error Handling

- [ ] **Fix Silent Config Directory Creation Failures**
  - **Severity:** MEDIUM - Silent failures
  - **Files:** `client/src-tauri/src/config.rs:32-38`
  - **Issue:** .ok() discards errors, later saves fail mysteriously
  - **Fix:** Replace .ok() with proper error handling: .map_err(|e| anyhow!("Failed to create config dir: {}", e))?

- [ ] **Add Image Load Error Handler & Timeout**
  - **Severity:** MEDIUM - UI hangs on load failure
  - **Files:** `client/src/components/VideoBackground.tsx:12-19`
  - **Issue:** No onerror handler or timeout, imageLoaded stays false forever
  - **Fix:** Add img.onerror callback, add 5s timeout to give up

- [ ] **Cap Progress Percentage at 100%**
  - **Severity:** MEDIUM - UI glitch
  - **Files:** `client/src-tauri/src/encoder.rs:357-369`
  - **Issue:** FFmpeg can encode more frames than expected, percentage exceeds 100%
  - **Fix:** Add .min(100.0) to percentage calculation

- [ ] **Fix FFprobe Logging to Use log Crate**
  - **Severity:** MEDIUM - Poor diagnostics
  - **Files:** `ffmpeg/src/builders/ffprobe_builder.rs:269-291`
  - **Issue:** Uses eprintln!() instead of log crate, not captured in logs
  - **Fix:** Replace eprintln! with log::error! and log::debug!

- [ ] **Add Server Configuration Validation**
  - **Severity:** MEDIUM - Invalid state accepted
  - **Files:** `server/src/configuration/mod.rs:38-59`
  - **Issue:** No validation of port range, directory existence, FFmpeg paths
  - **Fix:** Add validation on load: port 1-65535, dirs exist/readable, FFmpeg executable, timeouts reasonable

- [ ] **Reduce Concurrent FFprobe Process Limit**
  - **Severity:** MEDIUM - Resource exhaustion
  - **Files:** `server/src/media_files/scanner.rs:168`
  - **Issue:** buffer_unordered(100) allows 100 concurrent ffprobe processes
  - **Fix:** Reduce to 10-20 or make configurable

- [ ] **Fix Zombie Process Risk on Stdout/Stderr Failure**
  - **Severity:** MEDIUM - Resource leak
  - **Files:** `ffmpeg/src/lib.rs:322-323`
  - **Issue:** If capture fails, spawned process isn't killed
  - **Fix:** Call child.kill() in error handler before returning

### TLS/Reverse Proxy Support

- [ ] **Document Reverse Proxy Requirement**
  - **Severity:** MEDIUM - Security docs
  - **Files:** `server/README.md` (create/update)
  - **Issue:** No documentation on reverse proxy setup
  - **Fix:** Add README section with nginx/caddy examples, warn about direct exposure

- [ ] **Add X-Forwarded Headers Support**
  - **Severity:** MEDIUM - Reverse proxy compatibility
  - **Files:** `server/src/lib.rs`, new middleware
  - **Issue:** Server doesn't respect X-Forwarded-Proto, X-Forwarded-For
  - **Fix:** Add trust_proxy config, middleware to extract real client info, log warning if bound to 0.0.0.0

---

## 🟢 LOW Priority / Quality of Life

### UI/UX Improvements

- [ ] **Reduce VideoBackground Progress Animation Duration**
  - **Severity:** LOW - Minor stuttering
  - **Files:** `client/src/components/VideoBackground.tsx:56-58`
  - **Issue:** 0.5s transition causes lag with frequent updates
  - **Fix:** Reduce to 0.2s or remove transition

### Code Quality & Maintainability

- [ ] **Use Platform-Specific Cache Directory for FFmpeg Binaries**
  - **Severity:** LOW - Hardcoded relative path
  - **Files:** `ffmpeg/src/lib.rs:13`
  - **Issue:** "meta/ffmpeg" depends on working directory
  - **Fix:** Use ~/.cache/ffnodes/ffmpeg (Linux), ~/Library/Caches/ffnodes/ffmpeg (macOS), %LOCALAPPDATA%\ffnodes\ffmpeg (Windows)

- [ ] **Add Retry Logic for FFprobe Transient Errors**
  - **Severity:** LOW - Legitimate files skipped
  - **Files:** `server/src/media_files/scanner.rs:129`
  - **Issue:** Transient errors cause permanent skip
  - **Fix:** Add exponential backoff retry (3 attempts)

- [ ] **Implement Total Media Files Count**
  - **Severity:** LOW - Incomplete monitoring
  - **Files:** `server/src/api/monitoring.rs:43`
  - **Issue:** TODO comment, returns hardcoded 0
  - **Fix:** Query SELECT COUNT(*) FROM media_files

- [ ] **Add Job Priority Re-evaluation**
  - **Severity:** LOW - Suboptimal distribution
  - **Files:** `server/src/jobs/queue.rs:137`
  - **Issue:** Priority calculated once, never updated
  - **Fix:** Periodic recalculation based on age, size, client availability

- [ ] **Fix FFmpeg Filter Chaining Escaping**
  - **Severity:** LOW - Edge case bugs
  - **Files:** `ffmpeg/src/builders/ffmpeg_builder.rs:607-632`
  - **Issue:** Directly appends filters without escaping special chars
  - **Fix:** Validate filter syntax, properly escape colons, brackets, commas

### Documentation & Developer Experience

- [ ] **Add Database Migration System**
  - **Severity:** LOW - Schema versioning
  - **Files:** `server/src/database/` (new)
  - **Issue:** No migration system for schema changes
  - **Fix:** Add sqlx migrations, version schema

- [ ] **Add Structured Logging for Monitoring**
  - **Severity:** LOW - Observability
  - **Files:** All server files
  - **Issue:** Logs not structured for parsing
  - **Fix:** Add structured fields (client_id, job_id, file_path) to log statements

- [ ] **Add Backward Compatibility Handling for Config**
  - **Severity:** LOW - Upgrade experience
  - **Files:** `client/src-tauri/src/config.rs`, `server/src/configuration/mod.rs`
  - **Issue:** Missing fields could cause errors on upgrade
  - **Fix:** Use Option<T> for new fields, provide defaults, document changes

---

## 📊 Implementation Progress Tracking

### Phase 1: Critical Security
- **Status:** Not Started
- **Items:** 5
- **Completed:** 0
- **Blocked:** 0

### Phase 2: User-Requested Features
- **Status:** Not Started
- **Items:** 4
- **Completed:** 0
- **Blocked:** 0

### Phase 3: Stability & Data Integrity
- **Status:** Not Started
- **Items:** 8
- **Completed:** 0
- **Blocked:** 0

### Phase 4: Bug Fixes & UX
- **Status:** Not Started
- **Items:** 11
- **Completed:** 0
- **Blocked:** 0

### Phase 5: Quality of Life
- **Status:** Not Started
- **Items:** 9
- **Completed:** 0
- **Blocked:** 0

---

## 🎯 Recommended Implementation Order

1. **Fix CurrentJob.tsx Progress Display** (Quick win, user-facing bug)
2. **Remove Card Gradient Variant** (Quick win, user-requested)
3. **Change App Start to Paused** (Quick win, user-requested)
4. **Remove Old /setup Route** (Quick win, cleanup)
5. **Implement JWT Authentication** (Critical security)
6. **Fix Path Traversal Vulnerability** (Critical security)
7. **Fix Job Assignment Race Conditions** (Critical data integrity)
8. **Add Transaction Isolation** (Critical data integrity)
9. **Implement Job State Persistence** (High priority feature)
10. **Implement FFmpeg Resume with -ss** (High priority feature)
11. Continue with remaining HIGH priority items
12. Continue with MEDIUM priority items
13. Continue with LOW priority items

---

## 📝 Notes

- **Priority Levels:**
  - CRITICAL: Blocks production deployment or causes data loss/security breach
  - HIGH: Important features or bugs affecting core functionality
  - MEDIUM: Bugs affecting edge cases or improving reliability
  - LOW: Nice-to-have improvements and polish

- **Testing Required:** All CRITICAL and HIGH items must have tests before merging
- **Code Review:** All security-related changes require thorough review
- **Documentation:** Update README and docs for all user-facing changes

---

## 🔗 Related Documents

- **Full Plan:** `C:\Users\drew_\.claude\plans\transient-puzzling-oasis.md`
- **CLAUDE.md:** `F:\JetBrains\RustRover\FFNodes\CLAUDE.md` (project instructions)

---

**Total Items:** 52
**Critical/High:** 17
**Medium:** 11
**Low:** 9
**User-Requested:** 4
**Estimated Effort:** ~3-4 weeks for CRITICAL+HIGH+User-Requested items
