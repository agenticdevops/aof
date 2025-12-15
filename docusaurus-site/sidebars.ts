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
      label: 'Guides',
      items: [
        'guides/deployment',
        'guides/local-testing',
        'guides/testing-mcp',
      ],
    },
    {
      type: 'category',
      label: 'Reference',
      items: [
        'reference/agent-spec',
        'reference/agentflow-spec',
        'reference/aofctl',
        'reference/aofctl-examples',
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
        'tutorials/incident-response',
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
