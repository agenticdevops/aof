import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */
const sidebars: SidebarsConfig = {
  // Main documentation sidebar
  docsSidebar: [
    {
      type: 'doc',
      id: 'intro',
      label: 'Introduction',
    },
    {
      type: 'doc',
      id: 'getting-started',
      label: 'Getting Started',
    },
    {
      type: 'doc',
      id: 'concepts',
      label: 'Core Concepts',
    },
    {
      type: 'category',
      label: 'Concepts',
      items: [
        'concepts/fleets',
        'concepts/resource-selection',
        'concepts/github-integration',
        'concepts/jira-integration',
        'concepts/whatsapp-integration',
        'concepts/teams-integration',
      ],
    },
    {
      type: 'category',
      label: 'Architecture',
      items: [
        'architecture/multi-model-consensus',
      ],
    },
    {
      type: 'category',
      label: 'Tools',
      items: [
        'tools/index',
        'tools/builtin-tools',
        'tools/mcp-integration',
        'tools/custom-tools',
      ],
    },
    {
      type: 'category',
      label: 'Guides',
      items: [
        'guides/quickstart-telegram',
        'guides/quickstart-whatsapp',
        'guides/quickstart-teams',
        'guides/approval-workflow',
        'guides/deployment',
        'guides/local-testing',
        'guides/testing-mcp',
      ],
    },
    {
      type: 'category',
      label: 'Resources',
      items: [
        'reference/agent-spec',
        'reference/agentflow-spec',
        'reference/fleet-spec',
        'reference/trigger-spec',
        'reference/context-spec',
      ],
    },
    {
      type: 'category',
      label: 'Reference',
      items: [
        'reference/daemon-config',
        'reference/aofctl',
        'reference/github-integration',
        'reference/jira-integration',
        'reference/whatsapp-integration',
        'reference/teams-integration',
      ],
    },
  ],

  // Tutorials sidebar
  tutorialSidebar: [
    {
      type: 'category',
      label: 'Tutorials',
      items: [
        'tutorials/first-agent',
        'tutorials/slack-bot',
        'tutorials/telegram-ops-bot',
        'tutorials/whatsapp-ops-bot',
        'tutorials/teams-ops-bot',
        {
          type: 'category',
          label: 'GitHub Automation',
          items: [
            'tutorials/pr-review-automation',
            'tutorials/github-automation',
          ],
        },
        {
          type: 'category',
          label: 'Jira Automation',
          items: [
            'tutorials/jira-automation',
          ],
        },
        'tutorials/incident-response',
        'tutorials/rca-fleet',
        'tutorials/deep-analysis-fleet',
        'tutorials/multi-model-rca-quickstart',
        'tutorials/multi-model-rca',
      ],
    },
  ],

  // Examples sidebar
  examplesSidebar: [
    {
      type: 'doc',
      id: 'examples/index',
      label: 'Examples Overview',
    },
  ],
};

export default sidebars;
