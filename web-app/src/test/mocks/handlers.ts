import { http, HttpResponse } from 'msw'

// Mock tools for discovery endpoint
const MOCK_TOOLS = [
  // Critical tools (always available)
  {
    id: 'kubectl',
    name: 'Kubernetes CLI',
    path: '/usr/local/bin/kubectl',
    version: 'v1.29.0',
    category: 'Kubectl',
    available: true,
  },
  {
    id: 'terraform',
    name: 'Infrastructure as Code',
    path: '/usr/local/bin/terraform',
    version: '1.6.0',
    category: 'Terraform',
    available: true,
  },
  {
    id: 'docker',
    name: 'Docker Container Platform',
    path: '/usr/bin/docker',
    version: '25.0.0',
    category: 'Docker',
    available: true,
  },
  {
    id: 'git',
    name: 'Version Control',
    path: '/usr/bin/git',
    version: '2.43.0',
    category: 'Git',
    available: true,
  },
  {
    id: 'aws',
    name: 'AWS Command Line Interface',
    path: '/usr/local/bin/aws',
    version: '2.13.30',
    category: 'AWS',
    available: true,
  },
  {
    id: 'helm',
    name: 'Kubernetes Package Manager',
    path: '/usr/local/bin/helm',
    version: '3.13.0',
    category: 'Helm',
    available: true,
  },
  // Optional tools (may or may not be available)
  {
    id: 'prometheus',
    name: 'Prometheus Monitoring',
    path: '/usr/local/bin/prometheus',
    version: null,
    category: 'Prometheus',
    available: false,
  },
  {
    id: 'vault',
    name: 'HashiCorp Vault',
    path: null,
    version: null,
    category: 'Custom',
    available: false,
  },
]

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

  // Tool discovery endpoint
  http.get('http://localhost:7777/api/config/tools/discover', () => {
    return HttpResponse.json(MOCK_TOOLS)
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
