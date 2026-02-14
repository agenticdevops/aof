import ws from 'k6/ws';
import { check, sleep } from 'k6';
import { Counter, Trend, Rate } from 'k6/metrics';

// Custom metrics
const eventsReceived = new Counter('events_received');
const eventLatency = new Trend('event_latency_ms');
const connectionTime = new Trend('ws_connecting');
const connectionErrors = new Rate('ws_connection_errors');

// Test configuration - staged ramp to 50 WebSocket clients
export const options = {
  stages: [
    { duration: '30s', target: 10 },  // Ramp up to 10 clients
    { duration: '1m', target: 50 },   // Ramp up to 50 clients
    { duration: '2m', target: 50 },   // Hold at 50 clients
    { duration: '30s', target: 0 },   // Ramp down to 0
  ],
  thresholds: {
    'events_received': ['count>700'],           // At least 700 events total (50 VUs * ~14 events over 5 min)
    'event_latency_ms': ['p(95)<100'],          // P95 latency under 100ms
    'ws_connecting': ['p(95)<500'],             // Connection time under 500ms
    'ws_connection_errors': ['rate<0.01'],      // Less than 1% connection errors
  },
};

export default function () {
  const url = 'ws://localhost:8080/ws';
  let connectionStart = Date.now();

  const res = ws.connect(url, {}, function (socket) {
    const connectionDuration = Date.now() - connectionStart;
    connectionTime.add(connectionDuration);

    socket.on('open', () => {
      console.log(`VU ${__VU}: WebSocket connected in ${connectionDuration}ms`);
    });

    socket.on('message', (data) => {
      try {
        const event = JSON.parse(data);

        // Calculate latency
        const now = Date.now();
        const eventTime = new Date(event.timestamp).getTime();
        const latency = Math.max(0, now - eventTime);

        eventsReceived.add(1);
        eventLatency.add(latency);

        // Validate event structure
        check(event, {
          'has agent_id': (e) => e.agent_id !== undefined,
          'has session_id': (e) => e.session_id !== undefined,
          'has activity': (e) => e.activity !== undefined,
          'has timestamp': (e) => e.timestamp !== undefined,
        });

        // Log periodically
        if (eventsReceived.value % 100 === 0) {
          console.log(`VU ${__VU}: Received ${eventsReceived.value} events, latency: ${latency}ms`);
        }
      } catch (e) {
        console.error(`VU ${__VU}: Failed to parse event:`, e);
      }
    });

    socket.on('close', () => {
      console.log(`VU ${__VU}: WebSocket connection closed`);
    });

    socket.on('error', (e) => {
      console.error(`VU ${__VU}: WebSocket error:`, e);
      connectionErrors.add(1);
    });

    // Keep connection open for the stage duration
    socket.setTimeout(() => {
      socket.close();
    }, 300000); // 5 minutes max
  });

  check(res, {
    'connection successful': (r) => {
      if (!r || r.status !== 101) {
        connectionErrors.add(1);
        return false;
      }
      return true;
    },
  });

  sleep(1);
}
