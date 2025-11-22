# FFNodes API Documentation

This document provides complete API specification for the FFNodes distributed encoding server.

## Base URL

```
http://localhost:8080/api
```

## Authentication

FFNodes uses GUID-based authentication. Clients must first perform a handshake with the server's GUID to receive an authentication token (client ID).

## Error Responses

All endpoints may return error responses in the following format:

```json
{
  "message": "Error description",
  "status": 400
}
```

In debug mode, errors include a `stacktrace` field with detailed error information.

### HTTP Status Codes

- `200 OK` - Request succeeded
- `204 No Content` - Request succeeded with no response body
- `400 Bad Request` - Invalid request format or parameters
- `401 Unauthorized` - Invalid authentication or client not connected
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error

---

## Authentication Endpoints

### POST /handshake

Register a new client with the server using GUID authentication.

**Request Body:**
```json
{
  "server_guid": "550e8400-e29b-41d4-a716-446655440000",
  "display_name": "My Workstation",
  "computer_name": "DESKTOP-ABC123"
}
```

**Request Fields:**
- `server_guid` (string, required) - Server GUID from configuration
- `display_name` (string, required) - Human-readable client name
- `computer_name` (string, required) - Computer hostname or identifier

**Response:** `200 OK`
```json
{
  "auth_token": "client-uuid-here",
  "ffmpeg_template": "-c:v h264{HWACCEL_CODE} -preset medium -crf 23 -i {INPUT} {OUTPUT}"
}
```

**Response Fields:**
- `auth_token` (string) - Client ID to use for subsequent requests
- `ffmpeg_template` (string) - FFmpeg command template with placeholders

**Error Responses:**
- `401 Unauthorized` - Invalid server GUID
- `500 Internal Server Error` - Failed to register client

**Example:**
```bash
curl -X POST http://localhost:8080/api/handshake \
  -H "Content-Type: application/json" \
  -d '{
    "server_guid": "550e8400-e29b-41d4-a716-446655440000",
    "display_name": "Encoding Node 1",
    "computer_name": "NODE-001"
  }'
```

---

## Job Management Endpoints

### POST /jobs/request/{client_id}

Request the next available encoding job from the queue.

**Path Parameters:**
- `client_id` (string, required) - Client authentication token from handshake

**Response:** `200 OK` (job available)
```json
{
  "input_path": "/media/videos/movie.mp4",
  "output_template": "/media/videos/movie.mp4.h264.mp4",
  "job": {
    "id": "job-uuid",
    "media_file_path": "/media/videos/movie.mp4",
    "status": "assigned",
    "priority": 15000000,
    "assigned_client": "client-uuid",
    "assigned_at": 1705315200,
    "created_at": 1705315100
  }
}
```

**Response:** `204 No Content` (no jobs available)

**Response Fields:**
- `input_path` (string) - Source video file path
- `output_template` (string) - Suggested output file path
- `job` (object) - Job details:
  - `id` (string) - Job UUID
  - `media_file_path` (string) - Source file path
  - `status` (string) - Job status: "assigned"
  - `priority` (i64) - Job priority score
  - `assigned_client` (string) - Client ID
  - `assigned_at` (i64) - Unix timestamp
  - `created_at` (i64) - Unix timestamp

**Error Responses:**
- `401 Unauthorized` - Client not connected
- `500 Internal Server Error` - Failed to retrieve job

**Example:**
```bash
curl -X POST http://localhost:8080/api/jobs/request/client-uuid-here
```

---

### POST /jobs/{job_id}/start

Mark a job as started (status changes from "assigned" to "in_progress").

**Path Parameters:**
- `job_id` (string, required) - Job UUID from request_job response

**Response:** `200 OK`

**Error Responses:**
- `404 Not Found` - Job not found
- `500 Internal Server Error` - Failed to start job

**Example:**
```bash
curl -X POST http://localhost:8080/api/jobs/job-uuid-here/start
```

---

### POST /jobs/{job_id}/progress

Update encoding progress for a job.

**Path Parameters:**
- `job_id` (string, required) - Job UUID

**Request Body:**
```json
{
  "frame": 1250,
  "fps": 45.2,
  "bitrate": "2500kbits/s",
  "speed": "1.5x"
}
```

**Request Fields:**
- `frame` (i64, required) - Current frame number
- `fps` (f64, required) - Current encoding speed (frames per second)
- `bitrate` (string, required) - Current output bitrate
- `speed` (string, required) - Encoding speed multiplier

**Response:** `200 OK`

**Error Responses:**
- `404 Not Found` - Job not found
- `500 Internal Server Error` - Failed to update progress

**Example:**
```bash
curl -X POST http://localhost:8080/api/jobs/job-uuid-here/progress \
  -H "Content-Type: application/json" \
  -d '{
    "frame": 5420,
    "fps": 52.3,
    "bitrate": "2850kbits/s",
    "speed": "1.8x"
  }'
```

---

### POST /jobs/{job_id}/complete

Mark a job as completed successfully.

**Path Parameters:**
- `job_id` (string, required) - Job UUID

**Request Body:**
```json
{
  "output_size": 125829120,
  "output_bitrate": 2500000
}
```

**Request Fields:**
- `output_size` (i64, required) - Output file size in bytes
- `output_bitrate` (i64, required) - Output bitrate in bits per second

**Response:** `200 OK`

**Error Responses:**
- `404 Not Found` - Job not found
- `500 Internal Server Error` - Failed to complete job

**Example:**
```bash
curl -X POST http://localhost:8080/api/jobs/job-uuid-here/complete \
  -H "Content-Type: application/json" \
  -d '{
    "output_size": 125829120,
    "output_bitrate": 2500000
  }'
```

---

### POST /jobs/{job_id}/fail

Mark a job as failed with error message.

**Path Parameters:**
- `job_id` (string, required) - Job UUID

**Request Body:**
```json
{
  "error": "FFmpeg process exited with code 1: Invalid codec parameters"
}
```

**Request Fields:**
- `error` (string, required) - Error message describing the failure

**Response:** `200 OK`

**Error Responses:**
- `404 Not Found` - Job not found
- `500 Internal Server Error` - Failed to mark job as failed

**Notes:**
- Failed jobs automatically increment retry_count
- Priority is recalculated: `priority = file_size × encoding_complexity / (retry_count + 1)`
- Job is automatically requeued as "pending"

**Example:**
```bash
curl -X POST http://localhost:8080/api/jobs/job-uuid-here/fail \
  -H "Content-Type: application/json" \
  -d '{
    "error": "FFmpeg process crashed: Out of memory"
  }'
```

---

### GET /jobs/active

Get all currently active (in_progress) encoding jobs.

**Response:** `200 OK`
```json
[
  {
    "id": "job-uuid-1",
    "media_file_path": "/media/videos/movie1.mp4",
    "status": "in_progress",
    "priority": 15000000,
    "assigned_client": "client-uuid-1",
    "assigned_at": 1705315200,
    "started_at": 1705315210,
    "created_at": 1705315100
  },
  {
    "id": "job-uuid-2",
    "media_file_path": "/media/videos/movie2.mkv",
    "status": "in_progress",
    "priority": 12500000,
    "assigned_client": "client-uuid-2",
    "assigned_at": 1705315250,
    "started_at": 1705315260,
    "created_at": 1705315150
  }
]
```

**Response Fields:**
Array of job objects with fields:
- `id` (string) - Job UUID
- `media_file_path` (string) - Source file path
- `status` (string) - "in_progress"
- `priority` (i64) - Job priority score
- `assigned_client` (string) - Client ID processing this job
- `assigned_at` (i64) - Unix timestamp when assigned
- `started_at` (i64) - Unix timestamp when started
- `created_at` (i64) - Unix timestamp when created

**Error Responses:**
- `500 Internal Server Error` - Failed to retrieve active jobs

**Example:**
```bash
curl http://localhost:8080/api/jobs/active
```

---

## Client Management Endpoints

### POST /heartbeat/{client_id}

Send heartbeat to maintain client connection and prevent timeout.

**Path Parameters:**
- `client_id` (string, required) - Client authentication token

**Response:** `200 OK`

**Error Responses:**
- `401 Unauthorized` - Client not found or disconnected
- `500 Internal Server Error` - Failed to update heartbeat

**Notes:**
- Clients should send heartbeats every 60 seconds
- Default timeout is 300 seconds (configurable)
- Jobs from timed-out clients are automatically reassigned

**Example:**
```bash
curl -X POST http://localhost:8080/api/heartbeat/client-uuid-here
```

---

## Monitoring Endpoints

### GET /status

Get overall system status and statistics.

**Response:** `200 OK`
```json
{
  "total_media_files": 1523,
  "pending_jobs": 45,
  "active_jobs": 8,
  "connected_clients": 3
}
```

**Response Fields:**
- `total_media_files` (i64) - Total media files in database
- `pending_jobs` (i64) - Jobs waiting to be assigned
- `active_jobs` (i64) - Jobs currently being processed
- `connected_clients` (usize) - Number of connected clients

**Error Responses:**
- `500 Internal Server Error` - Failed to retrieve status

**Example:**
```bash
curl http://localhost:8080/api/status
```

---

### GET /clients

Get all connected clients with their statuses.

**Response:** `200 OK`
```json
[
  {
    "id": "client-uuid-1",
    "display_name": "Encoding Node 1",
    "computer_name": "NODE-001",
    "connected_at": "2025-01-15T10:30:00Z",
    "last_heartbeat": "2025-01-15T10:35:45Z",
    "active_jobs": 2
  },
  {
    "id": "client-uuid-2",
    "display_name": "Workstation",
    "computer_name": "DESKTOP-ABC",
    "connected_at": "2025-01-15T10:32:00Z",
    "last_heartbeat": "2025-01-15T10:35:50Z",
    "active_jobs": 1
  }
]
```

**Response Fields:**
Array of client status objects:
- `id` (string) - Client UUID
- `display_name` (string) - Human-readable client name
- `computer_name` (string) - Computer hostname
- `connected_at` (string) - ISO 8601 timestamp
- `last_heartbeat` (string) - ISO 8601 timestamp
- `active_jobs` (usize) - Number of jobs currently assigned to this client

**Error Responses:**
- `500 Internal Server Error` - Failed to retrieve clients

**Example:**
```bash
curl http://localhost:8080/api/clients
```

---

## WebSocket Endpoints

### GET /ws/progress

Establish WebSocket connection for real-time progress updates.

**Protocol:** WebSocket (ws:// or wss://)

**Connection URL:**
```
ws://localhost:8080/api/ws/progress
```

**Message Format:**
All messages are JSON objects with a `type` field indicating the event type.

### Event Types

#### JobAssigned
Sent when a job is assigned to a client.

```json
{
  "type": "job_assigned",
  "job_id": "job-uuid",
  "client_id": "client-uuid",
  "media_file": "/media/videos/movie.mp4"
}
```

#### Progress
Sent when a client reports encoding progress.

```json
{
  "type": "progress",
  "job_id": "job-uuid",
  "client_id": "client-uuid",
  "frame": 5420,
  "fps": 52.3,
  "speed": "1.8x"
}
```

#### JobCompleted
Sent when a job completes successfully.

```json
{
  "type": "job_completed",
  "job_id": "job-uuid",
  "client_id": "client-uuid"
}
```

#### JobFailed
Sent when a job fails.

```json
{
  "type": "job_failed",
  "job_id": "job-uuid",
  "client_id": "client-uuid",
  "error": "FFmpeg process exited with code 1"
}
```

#### ClientConnected
Sent when a new client connects.

```json
{
  "type": "client_connected",
  "client_id": "client-uuid",
  "display_name": "Encoding Node 1"
}
```

#### ClientDisconnected
Sent when a client disconnects.

```json
{
  "type": "client_disconnected",
  "client_id": "client-uuid"
}
```

**Example (JavaScript):**
```javascript
const ws = new WebSocket('ws://localhost:8080/api/ws/progress');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);

  switch(data.type) {
    case 'progress':
      console.log(`Job ${data.job_id}: Frame ${data.frame}, ${data.fps} fps, ${data.speed}`);
      break;
    case 'job_completed':
      console.log(`Job ${data.job_id} completed by ${data.client_id}`);
      break;
    case 'job_failed':
      console.error(`Job ${data.job_id} failed: ${data.error}`);
      break;
  }
};
```

**Note:** The WebSocket implementation is basic and does not yet broadcast events. This is planned for future implementation.

---

## Data Models

### EncodingJob

```typescript
{
  id: string;                    // UUID
  media_file_path: string;       // File path
  status: "pending" | "assigned" | "in_progress" | "completed" | "failed";
  priority: number;              // i64
  assigned_client?: string;      // UUID (nullable)
  assigned_at?: number;          // Unix timestamp (nullable)
  started_at?: number;           // Unix timestamp (nullable)
  completed_at?: number;         // Unix timestamp (nullable)
  error_message?: string;        // (nullable)
  output_path?: string;          // (nullable)
  output_size?: number;          // i64 (nullable)
  output_bitrate?: number;       // i64 (nullable)
  created_at: number;            // Unix timestamp
}
```

### Client

```typescript
{
  id: string;                    // UUID
  display_name: string;
  computer_name: string;
  connected_at: number;          // Unix timestamp
  last_heartbeat: number;        // Unix timestamp
  disconnected_at?: number;      // Unix timestamp (nullable)
}
```

### ProgressUpdate

```typescript
{
  frame: number;                 // i64
  fps: number;                   // f64
  bitrate: string;
  speed: string;
}
```

### EncodingProgress

```typescript
{
  job_id: string;                // UUID
  frame: number;                 // i64
  fps: number;                   // f64
  bitrate: string;
  speed: string;
  updated_at: number;            // Unix timestamp
}
```

---

## Client Implementation Guide

### Typical Client Flow

1. **Connect to Server**
   ```
   POST /api/handshake
   ```
   Store `auth_token` and `ffmpeg_template`

2. **Start Heartbeat Loop** (every 60 seconds)
   ```
   POST /api/heartbeat/{client_id}
   ```

3. **Job Processing Loop**

   a. Request job:
   ```
   POST /api/jobs/request/{client_id}
   ```

   b. If 204 No Content, wait and retry

   c. If 200 OK with job:
   - Start job: `POST /api/jobs/{job_id}/start`
   - Parse FFmpeg template with hardware acceleration code
   - Execute FFmpeg process
   - Send progress updates every 1-5 seconds: `POST /api/jobs/{job_id}/progress`
   - On success: `POST /api/jobs/{job_id}/complete`
   - On failure: `POST /api/jobs/{job_id}/fail`

4. **Error Handling**
   - On 401 Unauthorized: Re-authenticate with handshake
   - On network errors: Retry with exponential backoff
   - On timeout: Continue heartbeats, jobs will be reassigned by server

### FFmpeg Template Usage

Template: `-c:v h264{HWACCEL_CODE} -preset medium -crf 23 -i {INPUT} {OUTPUT}`

**Replacements:**
- `{HWACCEL_CODE}`:
  - `_nvenc` for NVIDIA GPUs
  - `_amf` for AMD GPUs
  - `_qsv` for Intel Quick Sync
  - `` (empty string) for CPU encoding
- `{INPUT}`: Source file path from `input_path`
- `{OUTPUT}`: Output file path from `output_template`

**Example:**
```bash
# For NVIDIA GPU:
ffmpeg -c:v h264_nvenc -preset medium -crf 23 -i input.mp4 output.mp4

# For CPU:
ffmpeg -c:v h264 -preset medium -crf 23 -i input.mp4 output.mp4
```

---

## Rate Limits and Timeouts

- **Client Timeout**: 300 seconds (default, configurable)
- **Heartbeat Interval**: Recommended 60 seconds
- **Progress Update Interval**: Recommended 1-5 seconds
- **Job Request Polling**: Recommended 5-10 seconds when queue is empty
- **Stale Job Check**: Every 60 seconds (server-side)

---

## Database

The server uses SQLite with WAL (Write-Ahead Logging) mode for concurrent access.

**Database File:** `app.db`

**Tables:**
- `media_files` - Media file metadata
- `encoding_jobs` - Job queue
- `clients` - Connected clients
- `encoding_progress` - Real-time progress tracking

For schema details, see [README.md](README.md#database-schema).

---

## Development and Testing

### Testing with curl

**Complete workflow example:**

```bash
# 1. Handshake
RESPONSE=$(curl -s -X POST http://localhost:8080/api/handshake \
  -H "Content-Type: application/json" \
  -d '{
    "server_guid": "YOUR-SERVER-GUID",
    "display_name": "Test Client",
    "computer_name": "TEST-MACHINE"
  }')

CLIENT_ID=$(echo $RESPONSE | jq -r '.auth_token')
echo "Client ID: $CLIENT_ID"

# 2. Request job
JOB_RESPONSE=$(curl -s -X POST http://localhost:8080/api/jobs/request/$CLIENT_ID)
JOB_ID=$(echo $JOB_RESPONSE | jq -r '.job.id')
echo "Job ID: $JOB_ID"

# 3. Start job
curl -X POST http://localhost:8080/api/jobs/$JOB_ID/start

# 4. Send progress
curl -X POST http://localhost:8080/api/jobs/$JOB_ID/progress \
  -H "Content-Type: application/json" \
  -d '{
    "frame": 1000,
    "fps": 45.0,
    "bitrate": "2500kbits/s",
    "speed": "1.5x"
  }'

# 5. Complete job
curl -X POST http://localhost:8080/api/jobs/$JOB_ID/complete \
  -H "Content-Type: application/json" \
  -d '{
    "output_size": 125829120,
    "output_bitrate": 2500000
  }'

# 6. Check status
curl http://localhost:8080/api/status
```

---

## FAQ

**Q: What happens if a client crashes during encoding?**
A: The server's job scheduler checks for stale jobs every 60 seconds. If a job has been assigned for longer than the timeout period (default 300 seconds) without progress, it's automatically requeued for another client.

**Q: Can multiple clients encode the same file?**
A: No. Each media file creates one job in the queue, and jobs are exclusively assigned to a single client at a time.

**Q: What happens to failed jobs?**
A: Failed jobs are automatically requeued with incremented retry_count. The priority is recalculated to deprioritize repeatedly failing jobs: `priority = file_size × encoding_complexity / (retry_count + 1)`

**Q: How does priority calculation work?**
A: Priority is calculated as: `file_size × encoding_complexity / (retry_count + 1)`, where `encoding_complexity = (width × height) × bitrate × duration`. Larger, more complex files have higher priority. Each retry decreases priority.

**Q: Can I customize the FFmpeg template?**
A: Yes, edit the `ffmpeg_template` field in `config.json`. The template must include `{HWACCEL_CODE}`, `{INPUT}`, and `{OUTPUT}` placeholders.

**Q: Is the WebSocket endpoint functional?**
A: The WebSocket connection is established but event broadcasting is not yet implemented. This is planned for a future update.

**Q: How do I find the server GUID?**
A: The server logs the GUID on startup: `Server GUID: 550e8400-e29b-41d4-a716-446655440000`. It's also stored in `config.json`.

---

## Support

For issues, feature requests, or contributions, visit the GitHub repository:
https://github.com/Drew-Chase/FFNodes
