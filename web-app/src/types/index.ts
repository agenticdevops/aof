// Type definitions for AOF Web App
// Add shared types and interfaces here

export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

export interface User {
  id: string
  email: string
  name: string
}
