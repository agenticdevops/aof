import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'Agentic Ops Framework',
  tagline: 'AI-Powered Automation for DevOps, SRE, and Platform Engineering',
  favicon: 'img/favicon.ico',

  // Future flags, see https://docusaurus.io/docs/api/docusaurus-config#future
  future: {
    v4: true, // Improve compatibility with the upcoming Docusaurus v4
  },

  // Set the production url of your site here
  url: 'https://docs.aof.sh',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'agenticdevops', // Usually your GitHub org/user name.
  projectName: 'aof', // Usually your repo name.

  onBrokenLinks: 'throw',

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          // Use main docs/ folder as single source of truth
          path: '../docs',
          sidebarPath: './sidebars.ts',
          // Exclude internal/technical docs not meant for public site
          exclude: [
            'LLM_*.md',
            'RUVECTOR_*.md',
            'DOCUMENTATION_INDEX.md',
            'architecture/**',
            'schemas/**',
            'agentflow/**',
          ],
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            'https://github.com/agenticdevops/aof/tree/main/',
        },
        blog: false, // Disable blog for now
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    // Replace with your project's social card
    image: 'img/aof-social-card.jpg',
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'AOF',
      logo: {
        alt: 'Agentic Ops Framework Logo',
        src: 'img/aof.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Documentation',
        },
        {
          type: 'docSidebar',
          sidebarId: 'tutorialSidebar',
          position: 'left',
          label: 'Tutorials',
        },
        {
          type: 'docSidebar',
          sidebarId: 'examplesSidebar',
          position: 'left',
          label: 'Examples',
        },
        {
          href: 'https://github.com/agenticdevops/aof',
          position: 'right',
          className: 'header-github-link',
          'aria-label': 'GitHub repository',
        },
      ],
    },
    announcementBar: {
      id: 'star_us',
      content:
        '⭐ If you like AOF, give us a star on <a target="_blank" rel="noopener noreferrer" href="https://github.com/agenticdevops/aof">GitHub</a>! It helps us reach more developers.',
      backgroundColor: '#1a1a2e',
      textColor: '#ffffff',
      isCloseable: true,
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Documentation',
          items: [
            {
              label: 'Getting Started',
              to: '/docs/getting-started',
            },
            {
              label: 'Core Concepts',
              to: '/docs/concepts',
            },
            {
              label: 'Tutorials',
              to: '/docs/tutorials/first-agent',
            },
            {
              label: 'Examples',
              to: '/docs/examples',
            },
          ],
        },
        {
          title: 'Reference',
          items: [
            {
              label: 'Agent Spec',
              to: '/docs/reference/agent-spec',
            },
            {
              label: 'AgentFlow Spec',
              to: '/docs/reference/agentflow-spec',
            },
            {
              label: 'CLI Reference',
              to: '/docs/reference/aofctl',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/agenticdevops/aof',
            },
            {
              label: 'Issues',
              href: 'https://github.com/agenticdevops/aof/issues',
            },
            {
              label: 'Discussions',
              href: 'https://github.com/agenticdevops/aof/discussions',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} OpsFlow LLC. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
