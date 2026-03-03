import { describe, it, expect, beforeEach } from 'vitest'
import { useInstallStore } from '../installStore'
import type { InstallStage } from '@/types'

describe('installStore', () => {
  beforeEach(() => {
    // 重置 store 状态到初始值
    useInstallStore.setState({
      installStatus: { type: 'NotInstalled' },
      progress: null,
      installMethod: 'online',
      logs: [],
      systemChecks: [],
      wizardStep: 0,
      offlinePackagePath: null,
      targetVersion: 'latest',
      isInstalling: false,
    })
  })

  describe('progress', () => {
    it('should set progress', () => {
      const { setProgress } = useInstallStore.getState()

      setProgress({
        stage: 'Installing' as InstallStage,
        percentage: 50,
        message: '正在安装...',
      })

      const { progress } = useInstallStore.getState()
      expect(progress).not.toBeNull()
      expect(progress?.stage).toBe('Installing')
      expect(progress?.percentage).toBe(50)
      expect(progress?.message).toBe('正在安装...')
    })

    it('should update progress', () => {
      const { setProgress } = useInstallStore.getState()

      setProgress({
        stage: 'Checking' as InstallStage,
        percentage: 10,
        message: '检查中...',
      })

      setProgress({
        stage: 'Complete' as InstallStage,
        percentage: 100,
        message: '完成',
      })

      const { progress } = useInstallStore.getState()
      expect(progress?.stage).toBe('Complete')
      expect(progress?.percentage).toBe(100)
    })

    it('should clear progress by setting null', () => {
      const { setProgress } = useInstallStore.getState()

      setProgress({
        stage: 'Installing' as InstallStage,
        percentage: 50,
        message: '安装中',
      })

      // 通过设置 null 来清除进度
      setProgress(null)

      const { progress } = useInstallStore.getState()
      expect(progress).toBeNull()
    })
  })

  describe('logs', () => {
    it('should add log entry', () => {
      const { addLog } = useInstallStore.getState()

      addLog('测试消息', 'info')

      const { logs } = useInstallStore.getState()
      expect(logs).toHaveLength(1)
      expect(logs[0].message).toBe('测试消息')
      expect(logs[0].level).toBe('info')
      expect(logs[0].timestamp).toBeDefined()
    })

    it('should add multiple logs', () => {
      const { addLog } = useInstallStore.getState()

      addLog('消息1', 'info')
      addLog('消息2', 'success')
      addLog('消息3', 'error')

      const { logs } = useInstallStore.getState()
      expect(logs).toHaveLength(3)
      expect(logs[0].level).toBe('info')
      expect(logs[1].level).toBe('success')
      expect(logs[2].level).toBe('error')
    })

    it('should clear logs', () => {
      const { addLog, clearLogs } = useInstallStore.getState()

      addLog('消息1', 'info')
      addLog('消息2', 'info')

      clearLogs()

      const { logs } = useInstallStore.getState()
      expect(logs).toHaveLength(0)
    })

    it('should limit max logs', () => {
      const { addLog } = useInstallStore.getState()

      // 添加超过最大限制的日志
      for (let i = 0; i < 150; i++) {
        addLog(`消息${i}`, 'info')
      }

      const { logs } = useInstallStore.getState()
      expect(logs.length).toBeLessThanOrEqual(100)
    })
  })

  describe('wizard step', () => {
    it('should set wizard step', () => {
      const { setWizardStep } = useInstallStore.getState()

      setWizardStep(2)

      const { wizardStep } = useInstallStore.getState()
      expect(wizardStep).toBe(2)
    })

    it('should increment wizard step', () => {
      const { setWizardStep } = useInstallStore.getState()

      setWizardStep(0)
      setWizardStep(1)
      setWizardStep(2)

      const { wizardStep } = useInstallStore.getState()
      expect(wizardStep).toBe(2)
    })
  })

  describe('install method', () => {
    it('should set install method', () => {
      const { setInstallMethod } = useInstallStore.getState()

      setInstallMethod('oneclick' as const)

      const { installMethod } = useInstallStore.getState()
      expect(installMethod).toBe('oneclick')
    })

    it('should change install method', () => {
      const { setInstallMethod } = useInstallStore.getState()

      // 初始值是 'online'，切换到 'oneclick'
      setInstallMethod('oneclick' as const)

      const { installMethod } = useInstallStore.getState()
      expect(installMethod).toBe('oneclick')
    })
  })

  describe('reset', () => {
    it('should reset all state', () => {
      const { setProgress, addLog, setWizardStep, setInstallMethod, reset } = useInstallStore.getState()

      // 设置一些状态
      setProgress({
        stage: 'Installing' as InstallStage,
        percentage: 50,
        message: '安装中',
      })
      addLog('日志消息', 'info')
      setWizardStep(2)
      setInstallMethod('oneclick' as const)

      // 重置
      reset()

      // 验证状态已重置
      const state = useInstallStore.getState()
      expect(state.progress).toBeNull()
      expect(state.logs).toHaveLength(0)
      expect(state.wizardStep).toBe(0)
      // reset 方法将 installMethod 重置为 'online'，不是 null
      expect(state.installMethod).toBe('online')
    })
  })

  describe('integration', () => {
    it('should simulate installation flow', () => {
      const { setProgress, addLog, setWizardStep, setInstallMethod } = useInstallStore.getState()

      // 模拟安装流程
      setWizardStep(0)
      setInstallMethod('oneclick' as const)

      addLog('开始安装', 'info')
      setProgress({ stage: 'Checking' as InstallStage, percentage: 0, message: '检查环境' })

      addLog('检查完成', 'success')
      setProgress({ stage: 'Installing' as InstallStage, percentage: 25, message: '正在解压' })

      addLog('解压完成', 'success')
      setProgress({ stage: 'Configuring' as InstallStage, percentage: 75, message: '配置中' })

      addLog('安装成功', 'success')
      setProgress({ stage: 'Complete' as InstallStage, percentage: 100, message: '完成' })

      // 验证最终状态
      const state = useInstallStore.getState()
      expect(state.progress?.percentage).toBe(100)
      // 一共添加了 4 条日志
      expect(state.logs).toHaveLength(4)
      expect(state.wizardStep).toBe(0)
      expect(state.installMethod).toBe('oneclick')
    })
  })
})
