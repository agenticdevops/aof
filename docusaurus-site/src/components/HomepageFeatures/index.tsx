import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import { Rocket, Wrench, Bot, Shield, Database, Zap } from 'lucide-react';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  description: ReactNode;
  Icon: React.ComponentType<{ size?: number; className?: string }>;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Quick to Deploy',
    Icon: Rocket,
    description: (
      <>
        Install with a single command and deploy your first AI agent in minutes.
        YAML-based configuration makes it easy to version control and share.
      </>
    ),
  },
  {
    title: 'Flexible Tools',
    Icon: Wrench,
    description: (
      <>
        Integrate with kubectl, shell commands, HTTP APIs, Slack, GitHub,
        PagerDuty, and custom MCP tools. Build agents that work with your
        existing infrastructure.
      </>
    ),
  },
  {
    title: 'Multi-Provider AI',
    Icon: Bot,
    description: (
      <>
        Use OpenAI, Anthropic, Ollama, or Groq models. Switch providers easily
        and run locally or in the cloud.
      </>
    ),
  },
  {
    title: 'Safe & Controlled',
    Icon: Shield,
    description: (
      <>
        Human-in-the-loop approvals, allowed command lists, and audit logging
        keep your infrastructure safe while enabling automation.
      </>
    ),
  },
  {
    title: 'Memory & Context',
    Icon: Database,
    description: (
      <>
        Persistent memory, RAG integration, and conversation history help agents
        learn and maintain context across interactions.
      </>
    ),
  },
  {
    title: 'Production Ready',
    Icon: Zap,
    description: (
      <>
        Built with Rust for performance and reliability. Supports fleets, workflows,
        scheduling, webhooks, and complex orchestration patterns.
      </>
    ),
  },
];

function Feature({title, description, Icon}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center padding-horiz--md">
        <div className={styles.featureIcon}>
          <Icon size={48} className={styles.featureSvg} />
        </div>
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
