/**
 * SquadCompositionPanel Component
 *
 * Main dashboard panel for viewing and managing agent squads.
 * Integrates BotTemplateSelector and SquadCompositionUI.
 */

import React, { useEffect, useState } from 'react'
import { useAppDispatch, useAppSelector } from '@/store'
import { listSquads, createSquadFromTemplate, deleteAgentFromSquad, deleteSquad } from '@/store/slices/configSlice'
import { BotTemplate } from '@/types/agents'
import { BotTemplateSelector } from './BotTemplateSelector'
import { SquadCompositionUI } from './SquadCompositionUI'

/**
 * Modal component for template selection
 */
interface ModalProps {
  isOpen: boolean
  onClose: () => void
  children: React.ReactNode
}

const Modal: React.FC<ModalProps> = ({ isOpen, onClose, children }) => {
  if (!isOpen) return null

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <button className="modal-close" onClick={onClose}>
          ✕
        </button>
        {children}
      </div>
    </div>
  )
}

/**
 * Empty state component
 */
interface EmptyStateProps {
  icon: string
  title: string
  description: string
  action: {
    label: string
    onClick: () => void
  }
}

const EmptyState: React.FC<EmptyStateProps> = ({ icon, title, description, action }) => {
  return (
    <div className="empty-state">
      <div className="empty-state-icon">{icon}</div>
      <h3>{title}</h3>
      <p>{description}</p>
      <button onClick={action.onClick} className="btn-primary">
        {action.label}
      </button>
    </div>
  )
}

/**
 * Loading spinner component
 */
interface LoadingSpinnerProps {
  message?: string
}

const LoadingSpinner: React.FC<LoadingSpinnerProps> = ({ message = 'Loading...' }) => {
  return (
    <div className="loading-spinner">
      <div className="spinner"></div>
      <p>{message}</p>
    </div>
  )
}

/**
 * SquadCompositionPanel Component
 *
 * Main component for squad management in configuration dashboard
 */
export const SquadCompositionPanel: React.FC = () => {
  const dispatch = useAppDispatch()
  const squads = useAppSelector(s => s.config.squads)
  const loading = useAppSelector(s => s.config.isLoading)
  const error = useAppSelector(s => s.config.error)

  const [showTemplateSelector, setShowTemplateSelector] = useState(false)
  const [selectedSquadId, setSelectedSquadId] = useState<string | null>(null)

  // Load squads on mount
  useEffect(() => {
    dispatch(listSquads())
  }, [dispatch])

  const selectedSquad = squads.find(s => s.id === selectedSquadId)

  const handleSelectTemplate = async (template: BotTemplate) => {
    try {
      await dispatch(createSquadFromTemplate(template)).unwrap()
      setShowTemplateSelector(false)
      // Optional: Show toast notification
    } catch (err) {
      // Error is already in Redux state
      console.error('Failed to create squad:', err)
    }
  }

  const handleRemoveAgent = async (agentId: string) => {
    if (!selectedSquad) return

    try {
      await dispatch(deleteAgentFromSquad({ squadId: selectedSquad.id, agentId })).unwrap()
    } catch (err) {
      console.error('Failed to remove agent:', err)
    }
  }

  const handleDeleteSquad = async (squadId: string) => {
    if (!confirm('Are you sure you want to delete this squad?')) {
      return
    }

    try {
      await dispatch(deleteSquad(squadId)).unwrap()
      setSelectedSquadId(null)
    } catch (err) {
      console.error('Failed to delete squad:', err)
    }
  }

  return (
    <div className="squad-composition-panel">
      {/* Panel Header */}
      <div className="panel-header">
        <div className="header-content">
          <h2>Agent Squads</h2>
          <p className="header-subtitle">Create and manage specialist agent teams</p>
        </div>
        <button
          onClick={() => setShowTemplateSelector(true)}
          className="btn-primary"
          disabled={loading}
        >
          + Add Squad
        </button>
      </div>

      {/* Error Alert */}
      {error && (
        <div className="alert alert-error" role="alert">
          <span className="alert-icon">⚠️</span>
          <div>
            <strong>Error</strong>
            <p>{error}</p>
          </div>
        </div>
      )}

      {/* Loading State */}
      {loading && squads.length === 0 && (
        <LoadingSpinner message="Loading squads..." />
      )}

      {/* Empty State */}
      {!loading && squads.length === 0 && (
        <EmptyState
          icon="👥"
          title="No squads yet"
          description="Start with a specialist squad template to deploy your first team of AI agents"
          action={{
            label: 'Add Your First Squad',
            onClick: () => setShowTemplateSelector(true)
          }}
        />
      )}

      {/* Squads List and Details */}
      {squads.length > 0 && (
        <div className="squads-container">
          {/* Squads Sidebar */}
          <div className="squads-sidebar">
            <h3 className="sidebar-title">Your Squads</h3>
            <div className="squads-list">
              {squads.map(squad => (
                <div
                  key={squad.id}
                  className={`squad-item ${selectedSquadId === squad.id ? 'selected' : ''}`}
                  onClick={() => setSelectedSquadId(squad.id)}
                  role="button"
                  tabIndex={0}
                  onKeyDown={e => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      setSelectedSquadId(squad.id)
                    }
                  }}
                >
                  <div className="squad-item-content">
                    <h4 className="squad-name">{squad.name}</h4>
                    <span className="squad-meta">{squad.agents.length} agents</span>
                  </div>
                  {selectedSquadId === squad.id && (
                    <button
                      className="squad-delete"
                      onClick={e => {
                        e.stopPropagation()
                        handleDeleteSquad(squad.id)
                      }}
                      title="Delete squad"
                      aria-label={`Delete ${squad.name}`}
                    >
                      🗑️
                    </button>
                  )}
                </div>
              ))}
            </div>
          </div>

          {/* Squad Details */}
          <div className="squad-details">
            {selectedSquad ? (
              <SquadCompositionUI
                squad={selectedSquad}
                onRemoveAgent={handleRemoveAgent}
                readOnly={false}
              />
            ) : (
              <div className="no-selection">
                <p>Select a squad to view details</p>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Template Selector Modal */}
      <Modal isOpen={showTemplateSelector} onClose={() => setShowTemplateSelector(false)}>
        <BotTemplateSelector
          onSelectTemplate={handleSelectTemplate}
          onClose={() => setShowTemplateSelector(false)}
          showDescription={true}
        />
      </Modal>
    </div>
  )
}
