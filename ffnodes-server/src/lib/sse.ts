import type { DashboardEvent } from '../types/api';

/**
 * Create an SSE (Server-Sent Events) connection
 * @param url SSE endpoint URL
 * @param onMessage Callback for dashboard events
 * @param onError Callback for connection errors
 * @returns EventSource instance
 */
export function createSSEConnection(
  url: string,
  onMessage: (event: DashboardEvent) => void,
  onError?: (error: Event) => void
): EventSource {
  const eventSource = new EventSource(url);

  // Handle specific event types
  eventSource.addEventListener('job_started', (e: MessageEvent) => {
    try {
      const data = JSON.parse(e.data);
      onMessage({ event: 'job_started', ...data });
    } catch (err) {
      console.error('Failed to parse job_started event:', err);
    }
  });

  eventSource.addEventListener('job_completed', (e: MessageEvent) => {
    try {
      const data = JSON.parse(e.data);
      onMessage({ event: 'job_completed', ...data });
    } catch (err) {
      console.error('Failed to parse job_completed event:', err);
    }
  });

  eventSource.addEventListener('job_failed', (e: MessageEvent) => {
    try {
      const data = JSON.parse(e.data);
      onMessage({ event: 'job_failed', ...data });
    } catch (err) {
      console.error('Failed to parse job_failed event:', err);
    }
  });

  eventSource.addEventListener('client_connected', (e: MessageEvent) => {
    try {
      const data = JSON.parse(e.data);
      onMessage({ event: 'client_connected', ...data });
    } catch (err) {
      console.error('Failed to parse client_connected event:', err);
    }
  });

  eventSource.addEventListener('client_disconnected', (e: MessageEvent) => {
    try {
      const data = JSON.parse(e.data);
      onMessage({ event: 'client_disconnected', ...data });
    } catch (err) {
      console.error('Failed to parse client_disconnected event:', err);
    }
  });

  eventSource.addEventListener('progress_update', (e: MessageEvent) => {
    try {
      const data = JSON.parse(e.data);
      onMessage({ event: 'progress_update', ...data });
    } catch (err) {
      console.error('Failed to parse progress_update event:', err);
    }
  });

  // Handle errors
  eventSource.onerror = (e) => {
    console.error('SSE connection error:', e);
    onError?.(e);
  };

  return eventSource;
}

/**
 * Create an SSE connection with automatic reconnection
 * @param url SSE endpoint URL
 * @param onMessage Callback for dashboard events or generic SSE data
 * @param onConnectionChange Callback for connection status changes
 * @returns Cleanup function
 */
export function createReconnectingSSE(
  url: string,
  onMessage: (event: DashboardEvent | any) => void,
  onConnectionChange?: (connected: boolean) => void
): () => void {
  let eventSource: EventSource | null = null;
  let reconnectTimeout: number | null = null;
  let reconnectAttempts = 0;
  const maxReconnectDelay = 30000; // 30 seconds

  const connect = () => {
    eventSource = new EventSource(url);

    // Handle generic 'message' events (default SSE event type for scan progress)
    eventSource.onmessage = (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data);
        onMessage(data);
      } catch (err) {
        console.error('Failed to parse SSE message:', err);
      }
    };

    // Handle specific event types (existing dashboard events)
    eventSource.addEventListener('job_started', (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data);
        onMessage({ event: 'job_started', ...data });
      } catch (err) {
        console.error('Failed to parse job_started event:', err);
      }
    });

    eventSource.addEventListener('job_completed', (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data);
        onMessage({ event: 'job_completed', ...data });
      } catch (err) {
        console.error('Failed to parse job_completed event:', err);
      }
    });

    eventSource.addEventListener('job_failed', (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data);
        onMessage({ event: 'job_failed', ...data });
      } catch (err) {
        console.error('Failed to parse job_failed event:', err);
      }
    });

    eventSource.addEventListener('client_connected', (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data);
        onMessage({ event: 'client_connected', ...data });
      } catch (err) {
        console.error('Failed to parse client_connected event:', err);
      }
    });

    eventSource.addEventListener('client_disconnected', (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data);
        onMessage({ event: 'client_disconnected', ...data });
      } catch (err) {
        console.error('Failed to parse client_disconnected event:', err);
      }
    });

    eventSource.addEventListener('progress_update', (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data);
        onMessage({ event: 'progress_update', ...data });
      } catch (err) {
        console.error('Failed to parse progress_update event:', err);
      }
    });

    // Handle errors
    eventSource.onerror = (e) => {
      console.error('SSE connection error:', e);
      eventSource?.close();
      onConnectionChange?.(false);

      // Calculate exponential backoff delay
      const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), maxReconnectDelay);
      reconnectAttempts++;

      console.log(`SSE disconnected. Reconnecting in ${delay}ms...`);

      reconnectTimeout = setTimeout(() => {
        connect();
      }, delay);
    };

    eventSource.onopen = () => {
      console.log('SSE connected');
      reconnectAttempts = 0;
      onConnectionChange?.(true);
    };
  };

  connect();

  // Return cleanup function
  return () => {
    if (eventSource) {
      eventSource.close();
      eventSource = null;
    }
    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout);
      reconnectTimeout = null;
    }
  };
}
