/**
 * BotTemplateSelector Component
 *
 * Displays available specialist bot templates and allows users to select
 * and create squads from them.
 */

import React, { useState } from 'react'
import { BotTemplate } from '@/types/agents'
import { BOT_TEMPLATES } from '@/data/botTemplates'

interface BotTemplateSelectorProps {
  onSelectTemplate: (template: BotTemplate) => Promise<void>
  onClose?: () => void
  showDescription?: boolean
}

interface TemplateCardProps {
  template: BotTemplate
  selected: boolean
  onSelect: (template: BotTemplate) => void
  loading: boolean
  onApply: () => Promise<void>
}

/**
 * TemplateCard Sub-Component
 * Displays a single template card with icon, name, and key info
 */
const TemplateCard: React.FC<TemplateCardProps> = ({
  template,
  selected,
  onSelect,
  loading,
  onApply
}) => {
  const [applying, setApplying] = useState(false)

  const handleApply = async () => {
    setApplying(true)
    try {
      await onApply()
    } finally {
      setApplying(false)
    }
  }

  return (
    <div
      className={`template-card ${selected ? 'selected' : ''}`}
      onClick={() => onSelect(template)}
      role="button"
      tabIndex={0}
      onKeyDown={e => {
        if (e.key === 'Enter' || e.key === ' ') {
          onSelect(template)
        }
      }}
    >
      <div className="template-header">
        <div className="template-icon">{template.icon}</div>
        <div className="template-info">
          <h3 className="template-name">{template.name}</h3>
          <p className="template-description">{template.description}</p>
        </div>
      </div>

      <div className="template-meta">
        <span className="agents-count">
          <strong>{template.agents.length}</strong> agents
        </span>
        <span className="setup-time">
          <strong>{template.estimatedSetupTime}</strong> min setup
        </span>
      </div>

      <div className="tools-required">
        <span className="label">Required tools:</span>
        <div className="tool-tags">
          {template.requiredTools.slice(0, 3).map(tool => (
            <span key={tool} className="tool-tag">
              {tool}
            </span>
          ))}
          {template.requiredTools.length > 3 && (
            <span className="tool-tag more">+{template.requiredTools.length - 3}</span>
          )}
        </div>
      </div>

      {selected && (
        <button
          onClick={e => {
            e.stopPropagation()
            handleApply()
          }}
          disabled={loading || applying}
          className="btn-apply"
        >
          {applying ? 'Creating...' : 'Create Squad'}
        </button>
      )}
    </div>
  )
}

/**
 * TemplateDetailsPanel Sub-Component
 * Shows detailed information about selected template
 */
interface TemplateDetailsPanelProps {
  template: BotTemplate
  onApply: () => Promise<void>
  loading: boolean
}

const TemplateDetailsPanel: React.FC<TemplateDetailsPanelProps> = ({
  template,
  onApply,
  loading
}) => {
  const [applying, setApplying] = useState(false)

  const handleApply = async () => {
    setApplying(true)
    try {
      await onApply()
    } finally {
      setApplying(false)
    }
  }

  return (
    <div className="template-details-panel">
      <div className="details-content">
        <h3>About {template.name}</h3>

        <section className="section">
          <h4>Use Case</h4>
          <p>{template.useCase}</p>
        </section>

        <section className="section">
          <h4>Team Members ({template.agents.length})</h4>
          <div className="agents-list">
            {template.agents.map(agent => (
              <div key={agent.id} className="agent-item">
                <div className="agent-badge">
                  <span className="role">{agent.role}</span>
                </div>
                <div className="agent-details">
                  <h5>{agent.name}</h5>
                  <p className="persona">{agent.persona.personality}</p>
                  <div className="skills">
                    {agent.skills.slice(0, 2).map(skill => (
                      <span key={skill.id} className="skill-tag">
                        {skill.name}
                      </span>
                    ))}
                    {agent.skills.length > 2 && (
                      <span className="skill-tag more">+{agent.skills.length - 2}</span>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </section>

        <section className="section">
          <h4>Success Metrics</h4>
          <ul className="metrics-list">
            {template.successMetrics.map(metric => (
              <li key={metric.name}>
                <strong>{metric.name}:</strong> {metric.description}
                <span className="target">{metric.target}</span>
              </li>
            ))}
          </ul>
        </section>

        <section className="section">
          <h4>Coordination Pattern</h4>
          <p>{template.coordinationPattern.replace('-', ' ')}</p>
        </section>
      </div>

      <div className="details-actions">
        <button onClick={handleApply} disabled={loading || applying} className="btn-primary">
          {applying ? 'Creating Squad...' : 'Create This Squad'}
        </button>
      </div>
    </div>
  )
}

/**
 * BotTemplateSelector Component
 *
 * Main component for template selection and squad creation
 */
export const BotTemplateSelector: React.FC<BotTemplateSelectorProps> = ({
  onSelectTemplate,
  onClose,
  showDescription = true
}) => {
  const [selectedTemplate, setSelectedTemplate] = useState<BotTemplate | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleSelectTemplate = async (template: BotTemplate) => {
    setLoading(true)
    setError(null)

    try {
      await onSelectTemplate(template)
      setSelectedTemplate(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create squad')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="bot-template-selector">
      <div className="selector-header">
        <h2>Choose a Specialist Squad</h2>
        <p>Get started with pre-configured agent teams for common DevOps tasks</p>
        {onClose && (
          <button onClick={onClose} className="btn-close" aria-label="Close">
            ✕
          </button>
        )}
      </div>

      {error && (
        <div className="alert alert-error" role="alert">
          <span className="alert-icon">⚠️</span>
          <div>
            <strong>Error creating squad</strong>
            <p>{error}</p>
          </div>
        </div>
      )}

      <div className="templates-container">
        <div className="templates-grid">
          {BOT_TEMPLATES.map(template => (
            <TemplateCard
              key={template.id}
              template={template}
              selected={selectedTemplate?.id === template.id}
              onSelect={setSelectedTemplate}
              loading={loading}
              onApply={() => handleSelectTemplate(template)}
            />
          ))}
        </div>

        {selectedTemplate && showDescription && (
          <div className="template-details">
            <TemplateDetailsPanel
              template={selectedTemplate}
              onApply={() => handleSelectTemplate(selectedTemplate)}
              loading={loading}
            />
          </div>
        )}
      </div>
    </div>
  )
}
