import client from './client'

export const conversationAPI = {
  startSession: () => client.post('/api/conversation/session'),
  sendMessage: (sessionId: string, message: string) =>
    client.post(`/api/conversation/session/${sessionId}/message`, { message }),
  confirmAgent: (sessionId: string, agentData: any) =>
    client.post(`/api/conversation/session/${sessionId}/confirm`, agentData),
}
