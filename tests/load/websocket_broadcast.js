import ws from 'k6/ws';
import { check } from 'k6';
import { Counter, Trend } from 'k6/metrics';

// Custom metrics
const eventsReceived = new Counter('events_received');
const eventLatency = new Trend('event_latency_ms');

// Test configuration
export const options = {
  vus: 10,
  duration: '2m',
  thresholds: {
    'events_received': ['count>100'], // At least 100 events received across all VUs
    'event_latency_ms': ['p(95)<100'], // P95 latency under 100ms
  },
};

export default function () {
  const url = 'ws://localhost:8080/ws';

  const res = ws.connect(url, {}, function (socket) {
    socket.on('open', () => {
      console.log('WebSocket connection established');
    });

    socket.on('message', (data) => {
      try {
        const event = JSON.parse(data);

        // Calculate latency from event timestamp to receive time
        const now = Date.now();
        const eventTime = new Date(event.timestamp).getTime();
        const latency = now - eventTime;

        eventsReceived.add(1);
        eventLatency.add(latency);

        check(event, {
          'event has agent_id': (e) => e.agent_id !== undefined,
          'event has session_id': (e) => e.session_id !== undefined,
          'event has activity': (e) => e.activity !== undefined,
        });
      } catch (e) {
        console.error('Failed to parse event:', e);
      }
    });

    socket.on('close', () => {
      console.log('WebSocket connection closed');
    });

    socket.on('error', (e) => {
      console.error('WebSocket error:', e);
    });

    // Keep connection open for the test duration
    socket.setTimeout(() => {
      socket.close();
    }, 120000); // 2 minutes
  });

  check(res, {
    'WebSocket connection successful': (r) => r && r.status === 101,
  });
}
