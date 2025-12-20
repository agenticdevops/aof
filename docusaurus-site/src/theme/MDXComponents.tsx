import React from 'react';
import MDXComponents from '@theme-original/MDXComponents';
import Admonition from '@theme/Admonition';

// Star callout component that can be used in any MDX file
function StarCallout() {
  return (
    <Admonition type="tip" title="Like AOF? Give us a star!">
      If you find AOF useful, please{' '}
      <a href="https://github.com/agenticdevops/aof" target="_blank" rel="noopener noreferrer">
        star us on GitHub
      </a>
      . It helps us reach more developers and grow the community.
    </Admonition>
  );
}

export default {
  ...MDXComponents,
  StarCallout,
};
