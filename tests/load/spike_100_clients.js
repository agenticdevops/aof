import ws from 'k6/ws';
import { check, sleep } from 'k6';
import { Counter, Trend, Rate } from 'k6/metrics';

// Custom metrics
const eventsReceived = new Counter('events_received');
const eventLatency = new Trend('event_latency_ms');
const connectionTime = new Trend('ws_connecting');
const connectionErrors = new Rate('ws_connection_errors');
const httpReqFailed = new Rate('http_req_failed');

// Test configuration - spike traffic to test resilience
export const options = {
  stages: [
    { duration: '10s', target: 100 },  // Spike to 100 clients quickly
    { duration: '2m', target: 100 },   // Hold spike for 2 minutes
    { duration: '10s', target: 0 },    // Drop to 0 quickly
  ],
  thresholds: {
    'http_req_failed': ['rate<0.01'],       // Less than 1% HTTP failures
    'ws_connecting': ['p(95)<1000'],        // Connection time under 1 second
    'event_latency_ms': ['p(95)<200'],      // P95 latency under 200ms (relaxed for spike)
    'ws_connection_errors': ['rate<0.05'],  // Less than 5% connection errors
  },
};

export default function () {
  const url = 'ws://localhost:8080/ws';
  let connectionStart = Date.now();

  const res = ws.connect(url, {}, function (socket) {
    const connectionDuration = Date.now() - connectionStart;
    connectionTime.add(connectionDuration);

    socket.on('open', () => {
      console.log(`VU ${__VU}: Connected in ${connectionDuration}ms during spike`);
    });

    socket.on('message', (data) => {
      try {
        const event = JSON.parse(data);

        const now = Date.now();
        const eventTime = new Date(event.timestamp).getTime();
        const latency = Math.max(0, now - eventTime);

        eventsReceived.add(1);
        eventLatency.add(latency);

        check(event, {
          'event is valid': (e) => e.agent_id && e.session_id && e.activity,
        });
      } catch (e) {
        console.error(`VU ${__VU}: Parse error:`, e);
      }
    });

    socket.on('close', () => {
      console.log(`VU ${__VU}: Closed`);
    });

    socket.on('error', (e) => {
      console.error(`VU ${__VU}: Error:`, e);
      connectionErrors.add(1);
    });

    // Hold connection for spike duration
    socket.setTimeout(() => {
      socket.close();
    }, 140000); // 2min 20s
  });

  const connectionSuccessful = res && res.status === 101;
  check(res, {
    'connection successful': () => connectionSuccessful,
  });

  if (!connectionSuccessful) {
    connectionErrors.add(1);
    httpReqFailed.add(1);
  } else {
    httpReqFailed.add(0);
  }

  sleep(0.5);
}
