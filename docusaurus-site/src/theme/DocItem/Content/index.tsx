import React from 'react';
import Content from '@theme-original/DocItem/Content';
import type ContentType from '@theme/DocItem/Content';
import type {WrapperProps} from '@docusaurus/types';
import Admonition from '@theme/Admonition';

type Props = WrapperProps<typeof ContentType>;

export default function ContentWrapper(props: Props): React.JSX.Element {
  return (
    <>
      <Admonition type="tip" title="Like AOF? Give us a star!">
        If you find AOF useful, please{' '}
        <a href="https://github.com/agenticdevops/aof" target="_blank" rel="noopener noreferrer">
          star us on GitHub
        </a>
        . It helps us reach more developers and grow the community.
      </Admonition>
      <Content {...props} />
    </>
  );
}
