/**
 * Toast notification component for real-time event notifications
 *
 * Features:
 * - Auto-dismiss after configurable duration
 * - Manual close button
 * - Slide-in animation from top-right
 * - Type-based color variants (success/error/info/warning)
 * - Dark mode support
 */

import React, { useEffect, useState } from 'react'

export type ToastType = 'success' | 'error' | 'info' | 'warning'

export interface ToastProps {
  /** Toast message text */
  message: string
  /** Toast type determines color and icon */
  type: ToastType
  /** Auto-dismiss duration in milliseconds (default: 4000ms) */
  duration?: number
  /** Callback when toast is closed */
  onClose?: () => void
}

/**
 * Toast notification component
 */
export function Toast({ message, type, duration = 4000, onClose }: ToastProps) {
  const [isVisible, setIsVisible] = useState(true)
  const [isExiting, setIsExiting] = useState(false)

  useEffect(() => {
    // Auto-dismiss timer
    const timer = setTimeout(() => {
      handleClose()
    }, duration)

    return () => clearTimeout(timer)
  }, [duration])

  const handleClose = () => {
    setIsExiting(true)
    setTimeout(() => {
      setIsVisible(false)
      onClose?.()
    }, 200) // Match exit animation duration
  }

  if (!isVisible) return null

  // Type-based styling
  const typeStyles: Record<ToastType, string> = {
    success: 'bg-emerald-500 text-white dark:bg-emerald-600',
    error: 'bg-red-500 text-white dark:bg-red-600',
    info: 'bg-blue-500 text-white dark:bg-blue-600',
    warning: 'bg-yellow-500 text-gray-900 dark:bg-yellow-600 dark:text-gray-100',
  }

  // Type-based icons
  const typeIcons: Record<ToastType, string> = {
    success: '✓',
    error: '✕',
    info: 'ℹ',
    warning: '⚠',
  }

  return (
    <div
      className={`
        fixed top-4 right-4 z-50 w-80 sm:w-96
        rounded-lg shadow-lg p-4
        flex items-start gap-3
        ${typeStyles[type]}
        ${isExiting ? 'animate-toast-exit' : 'animate-toast-enter'}
      `}
      role="alert"
    >
      {/* Icon */}
      <div className="flex-shrink-0 text-xl font-bold">
        {typeIcons[type]}
      </div>

      {/* Message */}
      <div className="flex-1 text-sm font-medium">
        {message}
      </div>

      {/* Close button */}
      <button
        onClick={handleClose}
        className="flex-shrink-0 text-lg opacity-70 hover:opacity-100 transition-opacity"
        aria-label="Close notification"
      >
        ×
      </button>
    </div>
  )
}

/**
 * Toast container component to manage multiple toasts
 */
interface ToastContainerProps {
  toasts: Array<{ id: string; message: string; type: ToastType; duration?: number }>
  onDismiss: (id: string) => void
}

export function ToastContainer({ toasts, onDismiss }: ToastContainerProps) {
  return (
    <div className="fixed top-0 right-0 z-50 pointer-events-none">
      <div className="flex flex-col gap-2 p-4 pointer-events-auto">
        {toasts.map((toast) => (
          <Toast
            key={toast.id}
            message={toast.message}
            type={toast.type}
            duration={toast.duration}
            onClose={() => onDismiss(toast.id)}
          />
        ))}
      </div>
    </div>
  )
}

/**
 * Toast state manager (React Context approach)
 */
interface ToastContextValue {
  showToast: (message: string, type: ToastType, duration?: number) => void
}

const ToastContext = React.createContext<ToastContextValue | null>(null)

interface ToastData {
  id: string
  message: string
  type: ToastType
  duration?: number
}

/**
 * Toast provider component
 *
 * Usage:
 * ```tsx
 * <ToastProvider>
 *   <App />
 * </ToastProvider>
 * ```
 */
export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<ToastData[]>([])

  const showToast = (message: string, type: ToastType, duration = 4000) => {
    const id = `toast-${Date.now()}-${Math.random()}`
    setToasts((prev) => [...prev, { id, message, type, duration }])
  }

  const dismissToast = (id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id))
  }

  return (
    <ToastContext.Provider value={{ showToast }}>
      {children}
      <ToastContainer toasts={toasts} onDismiss={dismissToast} />
    </ToastContext.Provider>
  )
}

/**
 * Hook to access toast functionality
 *
 * Usage:
 * ```tsx
 * const { showToast } = useToast()
 * showToast('Agent connected!', 'success')
 * ```
 */
export function useToast() {
  const context = React.useContext(ToastContext)
  if (!context) {
    throw new Error('useToast must be used within ToastProvider')
  }
  return context
}

/**
 * Standalone helper function for simple toast usage
 * (Alternative to context-based approach)
 */
let toastCallback: ((message: string, type: ToastType, duration?: number) => void) | null = null

export function registerToastCallback(
  callback: (message: string, type: ToastType, duration?: number) => void
) {
  toastCallback = callback
}

export function showToast(message: string, type: ToastType, duration?: number) {
  if (toastCallback) {
    toastCallback(message, type, duration)
  } else {
    console.warn('Toast callback not registered. Wrap app with ToastProvider.')
  }
}
