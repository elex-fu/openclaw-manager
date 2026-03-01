import { describe, it, expect, beforeEach } from 'vitest'
import { useAppStore } from '../appStore'

describe('appStore', () => {
  beforeEach(() => {
    // 重置 store 状态
    useAppStore.setState({
      notifications: [],
      currentPage: 'dashboard',
      theme: 'system',
      sidebarOpen: true,
      isLoading: false,
      globalError: null,
    })
  })

  describe('notifications', () => {
    it('should add notification', () => {
      const { addNotification } = useAppStore.getState()
      
      addNotification({
        title: 'Test',
        message: 'Test message',
        type: 'info',
      })
      
      const { notifications } = useAppStore.getState()
      expect(notifications).toHaveLength(1)
      expect(notifications[0].title).toBe('Test')
      expect(notifications[0].message).toBe('Test message')
      expect(notifications[0].type).toBe('info')
      expect(notifications[0].id).toBeDefined()
      expect(notifications[0].timestamp).toBeDefined()
    })

    it('should remove notification', () => {
      const { addNotification, removeNotification } = useAppStore.getState()
      
      addNotification({
        title: 'Test',
        message: 'Test message',
        type: 'info',
      })
      
      const { notifications } = useAppStore.getState()
      const id = notifications[0].id
      
      removeNotification(id)
      
      const { notifications: updatedNotifications } = useAppStore.getState()
      expect(updatedNotifications).toHaveLength(0)
    })

    it('should clear all notifications', () => {
      const { addNotification, clearNotifications } = useAppStore.getState()
      
      addNotification({ title: 'Test 1', message: 'Message 1', type: 'info' })
      addNotification({ title: 'Test 2', message: 'Message 2', type: 'success' })
      
      clearNotifications()
      
      const { notifications } = useAppStore.getState()
      expect(notifications).toHaveLength(0)
    })

    it('should limit max notifications to 5', () => {
      const { addNotification } = useAppStore.getState()
      
      // 添加 6 个通知
      for (let i = 0; i < 6; i++) {
        addNotification({
          title: `Test ${i}`,
          message: `Message ${i}`,
          type: 'info',
        })
      }
      
      const { notifications } = useAppStore.getState()
      expect(notifications).toHaveLength(5)
    })
  })

  describe('theme', () => {
    it('should set theme', () => {
      const { setTheme } = useAppStore.getState()
      
      setTheme('dark')
      
      const { theme } = useAppStore.getState()
      expect(theme).toBe('dark')
    })
  })

  describe('sidebar', () => {
    it('should toggle sidebar', () => {
      const { toggleSidebar } = useAppStore.getState()
      
      toggleSidebar()
      
      const { sidebarOpen } = useAppStore.getState()
      expect(sidebarOpen).toBe(false)
    })

    it('should set sidebar open state', () => {
      const { setSidebarOpen } = useAppStore.getState()
      
      setSidebarOpen(false)
      
      const { sidebarOpen } = useAppStore.getState()
      expect(sidebarOpen).toBe(false)
    })
  })

  describe('loading state', () => {
    it('should set loading state', () => {
      const { setLoading } = useAppStore.getState()
      
      setLoading(true)
      
      const { isLoading } = useAppStore.getState()
      expect(isLoading).toBe(true)
    })
  })

  describe('global error', () => {
    it('should set global error', () => {
      const { setGlobalError } = useAppStore.getState()
      
      setGlobalError('Something went wrong')
      
      const { globalError } = useAppStore.getState()
      expect(globalError).toBe('Something went wrong')
    })
  })
})
