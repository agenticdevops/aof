// Agent types
export interface Agent {
  id: string
  name: string
  model: string
  type: 'analyst' | 'coordinator' | 'specialist' | 'orchestrator'
  instructions: string
  capabilities: string[]
  createdAt?: string
  updatedAt?: string
}

// Tool types
export interface Tool {
  id: string
  name: string
  description?: string
  type?: 'api' | 'integration' | 'script'
  status?: 'ready' | 'needs-config'
  createdAt?: string
  // Tool discovery properties
  path?: string
  version?: string | null
  category?: string
  available?: boolean
}

// Platform types
export type PlatformType = 'slack' | 'discord' | 'telegram' | 'whatsapp' | 'github' | 'jira'

export interface PlatformConfig {
  [key: string]: any
}

export interface Platform {
  id: string
  name: string
  type: PlatformType
  icon: string
  connected: boolean
  config?: PlatformConfig
  username?: string
  createdAt?: string
  updatedAt?: string
}

// Project types
export interface Project {
  id: string
  name: string
  description: string
  createdAt?: string
  updatedAt?: string
}

// Onboarding wizard types
export interface OnboardingData {
  currentStep: 1 | 2 | 3 | 4
  project: Partial<Project>
  agent: Partial<Agent>
  platforms: Record<PlatformType, Partial<Platform>>
}

// API Response types
export interface ApiResponse<T> {
  data: T
  status: 'success' | 'error'
  message?: string
}

export interface ApiError {
  message: string
  code: string
  details?: Record<string, any>
}

// Component prop types
export interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger'
  size?: 'sm' | 'md' | 'lg'
  loading?: boolean
  disabled?: boolean
  fullWidth?: boolean
  children: React.ReactNode
  onClick?: (e: React.MouseEvent<HTMLButtonElement>) => void
  className?: string
  type?: 'button' | 'submit' | 'reset'
  icon?: React.ReactNode
  iconPosition?: 'left' | 'right'
}

export interface InputProps {
  label?: string
  error?: string
  disabled?: boolean
  placeholder?: string
  value?: string
  onChange?: (e: React.ChangeEvent<HTMLInputElement>) => void
  type?: string
  required?: boolean
  helperText?: string
  icon?: React.ReactNode
  className?: string
  fullWidth?: boolean
}

export interface TextAreaProps {
  label?: string
  error?: string
  disabled?: boolean
  placeholder?: string
  value?: string
  onChange?: (e: React.ChangeEvent<HTMLTextAreaElement>) => void
  rows?: number
  required?: boolean
  helperText?: string
  className?: string
  fullWidth?: boolean
}

export interface SelectOption {
  value: string
  label: string
  disabled?: boolean
}

export interface SelectProps {
  label?: string
  error?: string
  disabled?: boolean
  placeholder?: string
  value?: string
  onChange?: (value: string) => void
  options: SelectOption[]
  required?: boolean
  helperText?: string
  fullWidth?: boolean
}

export interface BadgeProps {
  variant?: 'success' | 'error' | 'warning' | 'info' | 'neutral'
  size?: 'sm' | 'md'
  children: React.ReactNode
  className?: string
  icon?: React.ReactNode
}

export interface CardProps {
  children: React.ReactNode
  className?: string
  elevation?: 'flat' | 'lifted' | 'focused'
  clickable?: boolean
  onClick?: () => void
  hoverable?: boolean
}

export interface ModalProps {
  isOpen: boolean
  onClose: () => void
  title?: string
  children: React.ReactNode
  size?: 'sm' | 'md' | 'lg'
  showCloseButton?: boolean
  footer?: React.ReactNode
}

export interface ConfirmDialogProps {
  isOpen: boolean
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  isDangerous?: boolean
  isLoading?: boolean
  onConfirm: () => void
  onCancel: () => void
}

export interface FormFieldProps {
  label?: string
  error?: string
  required?: boolean
  helperText?: string
  children: React.ReactNode
  className?: string
}
