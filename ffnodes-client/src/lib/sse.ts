/**
 * Create an SSE connection with automatic reconnection
 * @param url SSE endpoint URL
 * @param onMessage Callback for SSE data
 * @param onConnectionChange Callback for connection status changes
 * @returns Cleanup function
 */
export function createReconnectingSSE(
  url: string,
  onMessage: (data: any) => void,
  onConnectionChange?: (connected: boolean) => void
): () => void {
  let eventSource: EventSource | null = null;
  let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
  let reconnectAttempts = 0;
  const maxReconnectDelay = 30000; // 30 seconds

  const connect = () => {
    eventSource = new EventSource(url);

    // Handle generic 'message' events (default SSE event type)
    eventSource.onmessage = (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data);
        onMessage(data);
      } catch (err) {
        console.error('Failed to parse SSE message:', err);
      }
    };

    // Handle errors
    eventSource.onerror = (e) => {
      console.error('SSE connection error:', {
        readyState: eventSource?.readyState,
        url: eventSource?.url,
        error: e,
        errorType: (e as any).type,
      });

      // Log readyState meaning
      const states = {
        0: 'CONNECTING',
        1: 'OPEN',
        2: 'CLOSED'
      };
      console.log('SSE State:', states[(eventSource?.readyState ?? 2) as 0 | 1 | 2]);

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
