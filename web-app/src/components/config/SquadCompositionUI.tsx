/**
 * SquadCompositionUI Component
 *
 * Displays and manages the composition of an agent squad.
 * Shows team members, their roles, personas, and skills.
 */

import React, { useState } from 'react'
import { SquadConfig, SquadAgent } from '@/types/agents'

interface SquadCompositionUIProps {
  squad: SquadConfig
  onAddAgent?: (agent: SquadAgent) => void
  onRemoveAgent?: (agentId: string) => void
  onUpdateAgent?: (agentId: string, updates: Partial<SquadAgent>) => void
  readOnly?: boolean
}

interface AgentCardProps {
  agent: SquadAgent
  expanded: boolean
  onToggleExpand: () => void
  onRemove?: () => void
  onUpdate?: (updates: Partial<SquadAgent>) => void
}

/**
 * Helper to get avatar emoji based on agent persona
 */
const getAvatarEmoji = (persona: SquadAgent['persona']): string => {
  // Map trait types to emojis
  if (persona.personality_traits?.includes('leadership')) return '👑'
  if (persona.personality_traits?.includes('analytical')) return '🧠'
  if (persona.personality_traits?.includes('investigative')) return '🔍'
  if (persona.personality_traits?.includes('detail-oriented')) return '🎯'
  if (persona.personality_traits?.includes('responsive')) return '⚡'
  if (persona.personality_traits?.includes('technical')) return '⚙️'
  return '🤖'
}

/**
 * AgentCard Sub-Component
 * Displays a single agent in the squad with expandable details
 */
const AgentCard: React.FC<AgentCardProps> = ({
  agent,
  expanded,
  onToggleExpand,
  onRemove,
  onUpdate
}) => {
  const [isRemoving, setIsRemoving] = useState(false)

  const handleRemove = async () => {
    if (confirm(`Remove ${agent.name} from squad?`)) {
      setIsRemoving(true)
      try {
        onRemove?.()
      } finally {
        setIsRemoving(false)
      }
    }
  }

  const statusColor =
    agent.status === 'active'
      ? '#10b981'
      : agent.status === 'inactive'
        ? '#9ca3af'
        : '#ef4444'

  return (
    <div className={`agent-card ${expanded ? 'expanded' : ''}`}>
      <div
        className="card-header"
        onClick={onToggleExpand}
        role="button"
        tabIndex={0}
        onKeyDown={e => {
          if (e.key === 'Enter' || e.key === ' ') {
            onToggleExpand()
          }
        }}
      >
        <div className="agent-identity">
          <div className="agent-avatar">{getAvatarEmoji(agent.persona)}</div>
          <div className="agent-info">
            <h4 className="agent-name">{agent.name}</h4>
            <div className="agent-badges">
              <span className="role-badge">{agent.role}</span>
              <span className="status-badge" style={{ backgroundColor: statusColor }}>
                {agent.status}
              </span>
            </div>
          </div>
        </div>
        <div className="expand-icon">{expanded ? '▼' : '▶'}</div>
      </div>

      {expanded && (
        <div className="card-details">
          <section className="persona-section">
            <h5>Personality</h5>
            <p>{agent.persona.personality}</p>
            {agent.persona.personality_traits && agent.persona.personality_traits.length > 0 && (
              <div className="traits">
                {agent.persona.personality_traits.map(trait => (
                  <span key={trait} className="trait-tag">
                    {trait}
                  </span>
                ))}
              </div>
            )}
          </section>

          <section className="communication-section">
            <h5>Communication Style</h5>
            <p>{agent.persona.communication_style}</p>
          </section>

          {agent.persona.can && agent.persona.can.length > 0 && (
            <section className="capabilities-section">
              <h5>Capabilities</h5>
              <ul className="capabilities-list">
                {agent.persona.can.map(cap => (
                  <li key={cap}>✅ {cap}</li>
                ))}
              </ul>
            </section>
          )}

          {agent.persona.cannot && agent.persona.cannot.length > 0 && (
            <section className="boundaries-section">
              <h5>Boundaries</h5>
              <ul className="boundaries-list">
                {agent.persona.cannot.map(bound => (
                  <li key={bound}>🚫 {bound}</li>
                ))}
              </ul>
            </section>
          )}

          <section className="skills-section">
            <h5>Skills ({agent.skills.length})</h5>
            <ul className="skills-list">
              {agent.skills.map(skill => (
                <li key={skill.id} className="skill-item">
                  <div className="skill-header">
                    <span className="skill-name">{skill.name}</span>
                    <span className="skill-category">{skill.category}</span>
                  </div>
                  <p className="skill-description">{skill.description}</p>
                  {skill.tools.length > 0 && (
                    <div className="skill-tools">
                      {skill.tools.map(tool => (
                        <span key={tool} className="tool-tag">
                          {tool}
                        </span>
                      ))}
                    </div>
                  )}
                  <div className="skill-confidence">
                    <div className="confidence-bar">
                      <div
                        className="confidence-fill"
                        style={{ width: `${skill.confidence * 100}%` }}
                      ></div>
                    </div>
                    <span className="confidence-text">{(skill.confidence * 100).toFixed(0)}%</span>
                  </div>
                </li>
              ))}
            </ul>
          </section>

          {onRemove && (
            <div className="card-actions">
              <button
                onClick={handleRemove}
                disabled={isRemoving}
                className="btn-remove"
                aria-label={`Remove ${agent.name} from squad`}
              >
                {isRemoving ? 'Removing...' : '❌ Remove from Squad'}
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

/**
 * SquadOverview Sub-Component
 * Shows squad-level information and statistics
 */
interface SquadOverviewProps {
  squad: SquadConfig
}

const SquadOverview: React.FC<SquadOverviewProps> = ({ squad }) => {
  const roleCount = {
    orchestrator: squad.agents.filter(a => a.role === 'orchestrator').length,
    specialist: squad.agents.filter(a => a.role === 'specialist').length,
    assistant: squad.agents.filter(a => a.role === 'assistant').length
  }

  const activeCount = squad.agents.filter(a => a.status === 'active').length

  return (
    <div className="squad-overview">
      <h3 className="squad-name">{squad.name}</h3>
      <div className="overview-grid">
        <div className="stat">
          <span className="stat-value">{squad.agents.length}</span>
          <span className="stat-label">Total Agents</span>
        </div>
        <div className="stat">
          <span className="stat-value">{activeCount}</span>
          <span className="stat-label">Active</span>
        </div>
        <div className="stat">
          <span className="stat-value">{roleCount.orchestrator}</span>
          <span className="stat-label">Orchestrators</span>
        </div>
        <div className="stat">
          <span className="stat-value">{roleCount.specialist}</span>
          <span className="stat-label">Specialists</span>
        </div>
      </div>
      <p className="squad-dates">
        Created: {new Date(squad.createdAt).toLocaleDateString()}
      </p>
    </div>
  )
}

/**
 * SquadCompositionUI Component
 *
 * Main component for viewing and managing squad composition
 */
export const SquadCompositionUI: React.FC<SquadCompositionUIProps> = ({
  squad,
  onAddAgent,
  onRemoveAgent,
  onUpdateAgent,
  readOnly = false
}) => {
  const [expandedAgent, setExpandedAgent] = useState<string | null>(null)

  return (
    <div className="squad-composition-ui">
      <SquadOverview squad={squad} />

      <div className="agents-section">
        <h3 className="section-title">Team Members</h3>
        <div className="agents-grid">
          {squad.agents.map(agent => (
            <AgentCard
              key={agent.id}
              agent={agent}
              expanded={expandedAgent === agent.id}
              onToggleExpand={() => setExpandedAgent(expandedAgent === agent.id ? null : agent.id)}
              onRemove={!readOnly ? () => onRemoveAgent?.(agent.id) : undefined}
              onUpdate={!readOnly ? updates => onUpdateAgent?.(agent.id, updates) : undefined}
            />
          ))}
        </div>
      </div>

      {!readOnly && onAddAgent && (
        <div className="add-agent-section">
          <button onClick={() => {}} className="btn-add-agent" disabled>
            + Add Agent
          </button>
          <p className="button-hint">Coming soon: Add custom agents to squad</p>
        </div>
      )}
    </div>
  )
}
