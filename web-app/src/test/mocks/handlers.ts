import { http, HttpResponse } from 'msw'

export const handlers = [
  http.get('http://localhost:7777/api/config/agents', () => {
    return HttpResponse.json([
      { id: '1', name: 'Test Agent', model: 'claude', type: 'analyst', instructions: 'Test agent', capabilities: [] },
    ])
  }),

  http.post('http://localhost:7777/api/config/agents', async ({ request }) => {
    const body = (await request.json()) as any
    return HttpResponse.json({ id: '2', ...body })
  }),

  http.put('http://localhost:7777/api/config/agents/:id', async ({ request, params }) => {
    const body = (await request.json()) as any
    return HttpResponse.json({ id: params.id, ...body })
  }),

  http.delete('http://localhost:7777/api/config/agents/:id', () => {
    return HttpResponse.json({ success: true })
  }),

  http.get('http://localhost:7777/api/config/tools', () => {
    return HttpResponse.json([])
  }),

  http.get('http://localhost:7777/api/config/platforms', () => {
    return HttpResponse.json([])
  }),

  http.post('http://localhost:7777/api/config/platforms/:platform/test', async ({ request }) => {
    const body = (await request.json()) as any
    return HttpResponse.json({ success: true, message: 'Test passed', config: body })
  }),

  http.get('http://localhost:7777/api/config/version', () => {
    return HttpResponse.json({ version: '0.1.0' })
  }),

  http.post('http://localhost:7777/api/conversation/session', () => {
    return HttpResponse.json({ sessionId: 'test-session-123' })
  }),

  http.post('http://localhost:7777/api/conversation/session/:sessionId/message', async ({ request }) => {
    const body = (await request.json()) as any
    return HttpResponse.json({ id: 'msg-123', ...body })
  }),

  http.post('http://localhost:7777/api/conversation/session/:sessionId/confirm', async ({ request }) => {
    const body = (await request.json()) as any
    return HttpResponse.json({ confirmed: true, agent: body })
  }),
]
