import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import Heading from '@theme/Heading';

import styles from './index.module.css';

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <Heading as="h1" className="hero__title">
          {siteConfig.title}
        </Heading>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div className={styles.buttons}>
          <Link
            className="button button--secondary button--lg"
            to="/docs/getting-started">
            Get Started in 5 Minutes
          </Link>
          <Link
            className="button button--outline button--secondary button--lg"
            to="/docs/examples/"
            style={{marginLeft: '10px'}}>
            View Examples
          </Link>
        </div>
      </div>
    </header>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title={`${siteConfig.title} - AI-Powered DevOps Automation`}
      description="Build AI-powered automation for Kubernetes, incident response, and infrastructure management with the Agentic Ops Framework">
      <HomepageHeader />
      <main>
        <HomepageFeatures />

        <section className={styles.quickStart}>
          <div className="container">
            <div className="row">
              <div className="col col--12">
                <Heading as="h2" className="text--center">
                  Build Your First Agent in Minutes
                </Heading>
                <div className={styles.codeBlock}>
                  <pre>
{`# Install AOF
curl -sSL https://aof.dev/install.sh | bash

# Create an agent
cat > k8s-agent.yaml <<EOF
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
spec:
  model: openai:gpt-4
  instructions: |
    You are a Kubernetes expert assistant.
    Help users manage their clusters.
  tools:
    - type: Shell
      config:
        allowed_commands: [kubectl, helm]
EOF

# Run it
aofctl agent run k8s-agent.yaml`}
                  </pre>
                </div>
                <div className="text--center" style={{marginTop: '2rem'}}>
                  <Link
                    className="button button--primary button--lg"
                    to="/docs/getting-started">
                    Read the Full Getting Started Guide →
                  </Link>
                </div>
              </div>
            </div>
          </div>
        </section>

        <section className={styles.useCases}>
          <div className="container">
            <Heading as="h2" className="text--center">
              Built for DevOps, SRE, and Platform Engineering
            </Heading>
            <div className="row">
              <div className="col col--4">
                <div className="card" style={{height: '100%'}}>
                  <div className="card__header">
                    <h3>Incident Response</h3>
                  </div>
                  <div className="card__body">
                    <p>
                      Automate diagnostics, remediation, and post-incident analysis.
                      Integrate with PagerDuty, Slack, and your monitoring stack.
                    </p>
                  </div>
                  <div className="card__footer">
                    <Link to="/docs/tutorials/incident-response">
                      Learn More →
                    </Link>
                  </div>
                </div>
              </div>
              <div className="col col--4">
                <div className="card" style={{height: '100%'}}>
                  <div className="card__header">
                    <h3>Kubernetes Management</h3>
                  </div>
                  <div className="card__body">
                    <p>
                      Build intelligent K8s assistants that understand your cluster,
                      troubleshoot issues, and execute safe operations.
                    </p>
                  </div>
                  <div className="card__footer">
                    <Link to="/docs/examples/">
                      View Examples →
                    </Link>
                  </div>
                </div>
              </div>
              <div className="col col--4">
                <div className="card" style={{height: '100%'}}>
                  <div className="card__header">
                    <h3>Workflow Automation</h3>
                  </div>
                  <div className="card__body">
                    <p>
                      Create multi-step workflows with AI agents that handle
                      scheduling, webhooks, and complex decision-making.
                    </p>
                  </div>
                  <div className="card__footer">
                    <Link to="/docs/reference/agentflow-spec">
                      View Spec →
                    </Link>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>
      </main>
    </Layout>
  );
}
