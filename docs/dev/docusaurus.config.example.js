// Example Docusaurus 3.x Configuration for AOF Internal Developer Docs
//
// This shows how to integrate the developer documentation into a Docusaurus site.
// Copy and adapt to your actual docusaurus.config.js in the docs/ directory root.
//
// Usage:
// 1. Install Docusaurus: npm install -D docusaurus@latest @docusaurus/core @docusaurus/preset-classic
// 2. Create docs/docusaurus.config.js (copy from this template)
// 3. Create docs/sidebars.js (or use the one in docs/dev/sidebar.js)
// 4. Run: npm run docs
//
// For more info: https://docusaurus.io/

const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/dracula');

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'AOF - Agentic Ops Framework',
  tagline: 'Rust framework for building humanized agentic applications',
  favicon: 'img/favicon.ico',

  // Set the production url of your site here
  url: 'https://docs.aof.sh',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'agenticdevops', // Usually your GitHub org/user name.
  projectName: 'aof', // Usually your repo name.

  onBrokenLinks: 'warn',
  onBrokenMarkdownLinks: 'warn',

  // Even if you don't use internalization, you can use this field to set useful
  // metadata like html lang. For example, if your site is Chinese, you may want
  // to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          // Please change this to your repo.
          editUrl: 'https://github.com/agenticdevops/aof/edit/main/docs/',
          path: '.',  // Docs are in this directory
          include: ['**/*.md'],
          exclude: ['node_modules/**', 'build/**', '.git/**'],
        },
        blog: false,  // Disable blog for technical docs
        theme: {
          customCss: require.resolve('./custom.css'),
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      // Replace with your project's social card
      image: 'img/social-card.png',
      navbar: {
        title: 'AOF Docs',
        logo: {
          alt: 'AOF Logo',
          src: 'img/logo.svg',
        },
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'developers',
            position: 'left',
            label: 'Developer Docs',
          },
          {
            href: 'https://github.com/agenticdevops/aof',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              {
                label: 'Architecture',
                to: '/docs/ARCHITECTURE',
              },
              {
                label: 'Phase 6: Conversational Config',
                to: '/docs/PHASE-6-IMPLEMENTATION-SUMMARY',
              },
            ],
          },
          {
            title: 'Community',
            items: [
              {
                label: 'GitHub Issues',
                href: 'https://github.com/agenticdevops/aof/issues',
              },
              {
                label: 'Discussions',
                href: 'https://github.com/agenticdevops/aof/discussions',
              },
            ],
          },
          {
            title: 'More',
            items: [
              {
                label: 'GitHub',
                href: 'https://github.com/agenticdevops/aof',
              },
              {
                label: 'Apache 2.0 License',
                href: 'https://github.com/agenticdevops/aof/blob/main/LICENSE',
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} AOF Contributors. Built with Docusaurus.`,
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
        additionalLanguages: ['rust', 'toml', 'bash', 'typescript', 'json'],
      },
    }),

  plugins: [
    [
      '@docusaurus/plugin-content-docs',
      {
        id: 'dev',
        path: 'dev',
        routeBasePath: 'dev',
        sidebarPath: require.resolve('./dev/sidebar.js'),
        editUrl: 'https://github.com/agenticdevops/aof/edit/main/docs/dev/',
      },
    ],
  ],
};

module.exports = config;
