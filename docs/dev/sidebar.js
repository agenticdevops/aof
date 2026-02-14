// Docusaurus sidebar configuration for internal developer documentation
// Usage: Import in docusaurus.config.js

module.exports = {
  developers: [
    {
      type: 'doc',
      id: 'INDEX',
      label: '📚 Developer Docs Index',
    },
    {
      type: 'category',
      label: '🏗️ Architecture & Design',
      items: [
        'ARCHITECTURE',
        'AGENTFLOW_DESIGN',
        'decision-logging',
        'prompt-composition',
        'resource-locking',
        'sandbox-isolation',
        'reliability-metrics',
      ],
    },
    {
      type: 'category',
      label: '📋 Phase Implementation',
      items: [
        {
          type: 'category',
          label: 'Phase 1: Event Infrastructure ✅',
          items: [
            'event-infrastructure',
          ],
        },
        {
          type: 'category',
          label: 'Phase 2: Real Ops Capabilities ✅',
          items: [
            'skills-platform',
            'incident-response',
          ],
        },
        {
          type: 'category',
          label: 'Phase 3: Messaging Gateway ✅',
          items: [
            {
              type: 'link',
              label: 'Gateway Specs (internal/design/)',
              href: '../internal/design/',
            },
          ],
        },
        {
          type: 'category',
          label: 'Phase 4: Mission Control UI ✅',
          items: [
            'persona-system',
            'persona-loaders',
            'persona-ui-components',
          ],
        },
        {
          type: 'category',
          label: 'Phase 5: Agent Personas ✅',
          items: [
            {
              type: 'doc',
              id: 'persona-system',
              label: 'Persona System (Phase 4+5)',
            },
          ],
        },
        {
          type: 'category',
          label: 'Phase 6: Conversational Configuration ✅',
          items: [
            'PHASE-6-IMPLEMENTATION-SUMMARY',
            'conversational-architecture',
            'conversation-api',
            'agent-generation-pipeline',
            'squad-templates',
          ],
        },
        {
          type: 'category',
          label: 'Phase 7: Coordination Protocols (Planning)',
          items: [
            {
              type: 'link',
              label: 'Coordination Specs',
              href: '../internal/design/PHASE3-GITOPS-CICD.md',
            },
          ],
        },
        {
          type: 'category',
          label: 'Phase 8: Production Readiness (Planning)',
          items: [
            {
              type: 'link',
              label: 'Coming Soon',
              href: '#',
            },
          ],
        },
      ],
    },
    {
      type: 'category',
      label: '🛠️ Core Subsystems',
      items: [
        {
          type: 'category',
          label: 'Agent & Skill Systems',
          items: [
            'persona-system',
            'skills-platform',
            'agent-generation-pipeline',
          ],
        },
        {
          type: 'category',
          label: 'Execution & Coordination',
          items: [
            'AGENTFLOW_DESIGN',
            'incident-response',
            'resource-locking',
          ],
        },
        {
          type: 'category',
          label: 'Networking & Events',
          items: [
            'event-infrastructure',
            'sandbox-isolation',
          ],
        },
        {
          type: 'category',
          label: 'Reliability & Observability',
          items: [
            'decision-logging',
            'reliability-metrics',
          ],
        },
      ],
    },
    {
      type: 'category',
      label: '🔄 Integration Guides',
      items: [
        {
          type: 'category',
          label: 'Conversational Interface',
          items: [
            'conversational-architecture',
            'conversation-api',
            'squad-templates',
            'agent-generation-pipeline',
          ],
        },
        {
          type: 'category',
          label: 'Testing Patterns',
          items: [
            {
              type: 'doc',
              id: 'conversational-architecture',
              label: 'MockModel Pattern (Phase 6)',
            },
          ],
        },
      ],
    },
    {
      type: 'category',
      label: '📖 Contributing',
      items: [
        'CONTRIBUTING',
        'TOOLS_DEVELOPMENT',
      ],
    },
  ],
};
