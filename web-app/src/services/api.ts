import axios from 'axios'

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:7777'

export const apiClient = axios.create({
  baseURL: API_URL,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Add request/response interceptors as needed
apiClient.interceptors.request.use((config) => {
  // Add auth token if available
  return config
})

apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    console.error('API Error:', error)
    return Promise.reject(error)
  }
)

export default apiClient
