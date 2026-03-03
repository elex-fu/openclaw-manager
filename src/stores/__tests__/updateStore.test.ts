import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useUpdateStore } from '../updateStore'
import type { UpdateInfo, UpdateProgress } from '@/lib/tauri-api'

const mockUpdateInfo = (overrides: Partial<UpdateInfo> = {}): UpdateInfo => ({
  version: '1.1.0',
  releaseDate: '2024-01-15',
  changelog: 'Bug fixes and improvements',
  downloadUrl: 'https://example.com/download',
  checksum: 'abc123',
  mandatory: false,
  minSupportedVersion: null,
  ...overrides,
})

const mockUpdateProgress = (overrides: Partial<UpdateProgress> = {}): UpdateProgress => ({
  stage: 'Downloading',
  percentage: 50,
  message: 'Downloading update...',
  canCancel: true,
  ...overrides,
})

describe('updateStore', () => {
  beforeEach(() => {
    // Reset store state to initial values
    useUpdateStore.setState({
      currentVersion: null,
      latestVersion: null,
      updateInfo: null,
      hasUpdate: false,
      isChecking: false,
      lastCheckTime: null,
      progress: null,
      updateStatus: 'idle',
      settings: {
        autoCheck: true,
        autoDownload: false,
        autoInstall: false,
        checkInterval: 24,
        notifyOnUpdate: true,
        skipVersion: null,
      },
      backups: [],
      isCreatingBackup: false,
      offlinePackagePath: null,
      error: null,
      errorDetails: null,
      canCancel: true,
      needsRestart: false,
      updateHistory: [],
    })
  })

  describe('initial state', () => {
    it('should have correct initial state', () => {
      const state = useUpdateStore.getState()

      expect(state.currentVersion).toBeNull()
      expect(state.latestVersion).toBeNull()
      expect(state.updateInfo).toBeNull()
      expect(state.hasUpdate).toBe(false)
      expect(state.isChecking).toBe(false)
      expect(state.lastCheckTime).toBeNull()
      expect(state.progress).toBeNull()
      expect(state.updateStatus).toBe('idle')
      expect(state.settings.autoCheck).toBe(true)
      expect(state.settings.autoDownload).toBe(false)
      expect(state.settings.autoInstall).toBe(false)
      expect(state.settings.checkInterval).toBe(24)
      expect(state.settings.notifyOnUpdate).toBe(true)
      expect(state.settings.skipVersion).toBeNull()
      expect(state.backups).toEqual([])
      expect(state.isCreatingBackup).toBe(false)
      expect(state.offlinePackagePath).toBeNull()
      expect(state.error).toBeNull()
      expect(state.errorDetails).toBeNull()
      expect(state.canCancel).toBe(true)
      expect(state.needsRestart).toBe(false)
      expect(state.updateHistory).toEqual([])
    })
  })

  describe('version info', () => {
    it('should set current version', () => {
      const { setCurrentVersion } = useUpdateStore.getState()

      setCurrentVersion('1.0.0')

      const { currentVersion } = useUpdateStore.getState()
      expect(currentVersion).toBe('1.0.0')
    })

    it('should set latest version', () => {
      const { setLatestVersion } = useUpdateStore.getState()

      setLatestVersion('1.1.0')

      const { latestVersion } = useUpdateStore.getState()
      expect(latestVersion).toBe('1.1.0')
    })

    it('should set update info', () => {
      const { setUpdateInfo } = useUpdateStore.getState()
      const info = mockUpdateInfo()

      setUpdateInfo(info)

      const { updateInfo } = useUpdateStore.getState()
      expect(updateInfo).toEqual(info)
    })

    it('should clear update info', () => {
      const { setUpdateInfo } = useUpdateStore.getState()

      setUpdateInfo(mockUpdateInfo())
      setUpdateInfo(null)

      const { updateInfo } = useUpdateStore.getState()
      expect(updateInfo).toBeNull()
    })
  })

  describe('update check status', () => {
    it('should set has update', () => {
      const { setHasUpdate } = useUpdateStore.getState()

      setHasUpdate(true)

      const { hasUpdate } = useUpdateStore.getState()
      expect(hasUpdate).toBe(true)
    })

    it('should set checking status', () => {
      const { setIsChecking } = useUpdateStore.getState()

      setIsChecking(true)

      const { isChecking } = useUpdateStore.getState()
      expect(isChecking).toBe(true)
    })

    it('should set last check time', () => {
      const { setLastCheckTime } = useUpdateStore.getState()
      const now = Date.now()

      setLastCheckTime(now)

      const { lastCheckTime } = useUpdateStore.getState()
      expect(lastCheckTime).toBe(now)
    })
  })

  describe('progress tracking', () => {
    it('should set progress', () => {
      const { setProgress } = useUpdateStore.getState()
      const progress = mockUpdateProgress()

      setProgress(progress)

      const { progress: storedProgress } = useUpdateStore.getState()
      expect(storedProgress).toEqual(progress)
    })

    it('should update progress partially', () => {
      const { setProgress, updateProgress } = useUpdateStore.getState()

      setProgress(mockUpdateProgress({ percentage: 0 }))
      updateProgress({ percentage: 75, message: 'Almost done' })

      const { progress } = useUpdateStore.getState()
      expect(progress?.percentage).toBe(75)
      expect(progress?.message).toBe('Almost done')
      expect(progress?.stage).toBe('Downloading') // unchanged
    })

    it('should handle updateProgress when progress is null', () => {
      const { updateProgress } = useUpdateStore.getState()

      // Should not throw error
      expect(() => updateProgress({ percentage: 50 })).not.toThrow()

      const { progress } = useUpdateStore.getState()
      expect(progress).toBeNull()
    })

    it('should clear progress', () => {
      const { setProgress } = useUpdateStore.getState()

      setProgress(mockUpdateProgress())
      setProgress(null)

      const { progress } = useUpdateStore.getState()
      expect(progress).toBeNull()
    })
  })

  describe('update status', () => {
    it('should set update status', () => {
      const { setUpdateStatus } = useUpdateStore.getState()

      setUpdateStatus('downloading')

      const { updateStatus } = useUpdateStore.getState()
      expect(updateStatus).toBe('downloading')
    })

    it('should set all valid update statuses', () => {
      const { setUpdateStatus } = useUpdateStore.getState()
      const statuses = ['idle', 'checking', 'downloading', 'installing', 'completed', 'error', 'rollback'] as const

      statuses.forEach((status) => {
        setUpdateStatus(status)
        expect(useUpdateStore.getState().updateStatus).toBe(status)
      })
    })
  })

  describe('settings', () => {
    it('should set all settings at once', () => {
      const { setSettings } = useUpdateStore.getState()

      setSettings({
        autoCheck: false,
        autoDownload: true,
        checkInterval: 48,
      })

      const { settings } = useUpdateStore.getState()
      expect(settings.autoCheck).toBe(false)
      expect(settings.autoDownload).toBe(true)
      expect(settings.checkInterval).toBe(48)
      expect(settings.autoInstall).toBe(false) // unchanged
    })

    it('should set auto check', () => {
      const { setAutoCheck } = useUpdateStore.getState()

      setAutoCheck(false)

      const { settings } = useUpdateStore.getState()
      expect(settings.autoCheck).toBe(false)
    })

    it('should set auto download', () => {
      const { setAutoDownload } = useUpdateStore.getState()

      setAutoDownload(true)

      const { settings } = useUpdateStore.getState()
      expect(settings.autoDownload).toBe(true)
    })

    it('should set auto install', () => {
      const { setAutoInstall } = useUpdateStore.getState()

      setAutoInstall(true)

      const { settings } = useUpdateStore.getState()
      expect(settings.autoInstall).toBe(true)
    })

    it('should set check interval', () => {
      const { setCheckInterval } = useUpdateStore.getState()

      setCheckInterval(12)

      const { settings } = useUpdateStore.getState()
      expect(settings.checkInterval).toBe(12)
    })

    it('should set notify on update', () => {
      const { setNotifyOnUpdate } = useUpdateStore.getState()

      setNotifyOnUpdate(false)

      const { settings } = useUpdateStore.getState()
      expect(settings.notifyOnUpdate).toBe(false)
    })

    it('should set skip version', () => {
      const { setSkipVersion } = useUpdateStore.getState()

      setSkipVersion('1.1.0')

      const { settings } = useUpdateStore.getState()
      expect(settings.skipVersion).toBe('1.1.0')
    })

    it('should clear skip version', () => {
      const { setSkipVersion } = useUpdateStore.getState()

      setSkipVersion('1.1.0')
      setSkipVersion(null)

      const { settings } = useUpdateStore.getState()
      expect(settings.skipVersion).toBeNull()
    })

    it('should set hasUpdate to false when skipping current latest version', () => {
      const { setLatestVersion, setHasUpdate, setSkipVersion } = useUpdateStore.getState()

      setLatestVersion('1.1.0')
      setHasUpdate(true)
      setSkipVersion('1.1.0')

      const { hasUpdate } = useUpdateStore.getState()
      expect(hasUpdate).toBe(false)
    })
  })

  describe('backup management', () => {
    it('should set backups', () => {
      const { setBackups } = useUpdateStore.getState()
      const backups = [
        { createdAt: '2024-01-01', version: '1.0.0', path: '/backup/1' },
        { createdAt: '2024-02-01', version: '1.1.0', path: '/backup/2' },
      ]

      setBackups(backups)

      const { backups: storedBackups } = useUpdateStore.getState()
      expect(storedBackups).toHaveLength(2)
    })

    it('should set creating backup status', () => {
      const { setIsCreatingBackup } = useUpdateStore.getState()

      setIsCreatingBackup(true)

      const { isCreatingBackup } = useUpdateStore.getState()
      expect(isCreatingBackup).toBe(true)
    })
  })

  describe('offline update', () => {
    it('should set offline package path', () => {
      const { setOfflinePackagePath } = useUpdateStore.getState()

      setOfflinePackagePath('/path/to/package.zip')

      const { offlinePackagePath } = useUpdateStore.getState()
      expect(offlinePackagePath).toBe('/path/to/package.zip')
    })

    it('should clear offline package path', () => {
      const { setOfflinePackagePath } = useUpdateStore.getState()

      setOfflinePackagePath('/path/to/package.zip')
      setOfflinePackagePath(null)

      const { offlinePackagePath } = useUpdateStore.getState()
      expect(offlinePackagePath).toBeNull()
    })
  })

  describe('error handling', () => {
    it('should set error', () => {
      const { setError } = useUpdateStore.getState()

      setError('Update failed')

      const { error } = useUpdateStore.getState()
      expect(error).toBe('Update failed')
    })

    it('should set error details', () => {
      const { setErrorDetails } = useUpdateStore.getState()

      setErrorDetails('Stack trace: ...')

      const { errorDetails } = useUpdateStore.getState()
      expect(errorDetails).toBe('Stack trace: ...')
    })

    it('should clear error', () => {
      const { setError, setErrorDetails } = useUpdateStore.getState()

      setError('Error')
      setErrorDetails('Details')
      setError(null)
      setErrorDetails(null)

      const { error, errorDetails } = useUpdateStore.getState()
      expect(error).toBeNull()
      expect(errorDetails).toBeNull()
    })
  })

  describe('cancel and restart states', () => {
    it('should set can cancel', () => {
      const { setCanCancel } = useUpdateStore.getState()

      setCanCancel(false)

      const { canCancel } = useUpdateStore.getState()
      expect(canCancel).toBe(false)
    })

    it('should set needs restart', () => {
      const { setNeedsRestart } = useUpdateStore.getState()

      setNeedsRestart(true)

      const { needsRestart } = useUpdateStore.getState()
      expect(needsRestart).toBe(true)
    })
  })

  describe('update history', () => {
    it('should add update history record', () => {
      const { addUpdateHistory } = useUpdateStore.getState()
      const record = {
        version: '1.1.0',
        timestamp: Date.now(),
        status: 'success' as const,
        message: 'Update successful',
      }

      addUpdateHistory(record)

      const { updateHistory } = useUpdateStore.getState()
      expect(updateHistory).toHaveLength(1)
      expect(updateHistory[0].version).toBe('1.1.0')
      expect(updateHistory[0].status).toBe('success')
    })

    it('should add multiple history records', () => {
      const { addUpdateHistory } = useUpdateStore.getState()

      addUpdateHistory({ version: '1.0.0', timestamp: 1000, status: 'success' })
      addUpdateHistory({ version: '1.1.0', timestamp: 2000, status: 'failed' })
      addUpdateHistory({ version: '1.2.0', timestamp: 3000, status: 'success' })

      const { updateHistory } = useUpdateStore.getState()
      expect(updateHistory).toHaveLength(3)
      // Most recent first
      expect(updateHistory[0].version).toBe('1.2.0')
    })

    it('should limit history to 50 records', () => {
      const { addUpdateHistory } = useUpdateStore.getState()

      // Add 55 records
      for (let i = 0; i < 55; i++) {
        addUpdateHistory({
          version: `1.${i}.0`,
          timestamp: i * 1000,
          status: 'success',
        })
      }

      const { updateHistory } = useUpdateStore.getState()
      expect(updateHistory).toHaveLength(50)
      // Most recent should be kept
      expect(updateHistory[0].version).toBe('1.54.0')
    })
  })

  describe('computed properties', () => {
    describe('isUpdateAvailable', () => {
      it('should return true when hasUpdate is true and updateInfo is available', () => {
        const { setHasUpdate, setUpdateInfo } = useUpdateStore.getState()

        setHasUpdate(true)
        setUpdateInfo(mockUpdateInfo({ version: '1.1.0' }))

        const { isUpdateAvailable } = useUpdateStore.getState()
        expect(isUpdateAvailable()).toBe(true)
      })

      it('should return false when hasUpdate is false', () => {
        const { setHasUpdate, setUpdateInfo } = useUpdateStore.getState()

        setHasUpdate(false)
        setUpdateInfo(mockUpdateInfo())

        const { isUpdateAvailable } = useUpdateStore.getState()
        expect(isUpdateAvailable()).toBe(false)
      })

      it('should return false when updateInfo is null', () => {
        const { setHasUpdate } = useUpdateStore.getState()

        setHasUpdate(true)

        const { isUpdateAvailable } = useUpdateStore.getState()
        expect(isUpdateAvailable()).toBe(false)
      })

      it('should return false when version is skipped', () => {
        const { setHasUpdate, setUpdateInfo, setSkipVersion } = useUpdateStore.getState()

        setHasUpdate(true)
        setUpdateInfo(mockUpdateInfo({ version: '1.1.0' }))
        setSkipVersion('1.1.0')

        const { isUpdateAvailable } = useUpdateStore.getState()
        expect(isUpdateAvailable()).toBe(false)
      })
    })

    describe('isUpdateInProgress', () => {
      it('should return true when status is downloading', () => {
        const { setUpdateStatus } = useUpdateStore.getState()

        setUpdateStatus('downloading')

        const { isUpdateInProgress } = useUpdateStore.getState()
        expect(isUpdateInProgress()).toBe(true)
      })

      it('should return true when status is installing', () => {
        const { setUpdateStatus } = useUpdateStore.getState()

        setUpdateStatus('installing')

        const { isUpdateInProgress } = useUpdateStore.getState()
        expect(isUpdateInProgress()).toBe(true)
      })

      it('should return false when status is idle', () => {
        const { setUpdateStatus } = useUpdateStore.getState()

        setUpdateStatus('idle')

        const { isUpdateInProgress } = useUpdateStore.getState()
        expect(isUpdateInProgress()).toBe(false)
      })

      it('should return false when status is completed', () => {
        const { setUpdateStatus } = useUpdateStore.getState()

        setUpdateStatus('completed')

        const { isUpdateInProgress } = useUpdateStore.getState()
        expect(isUpdateInProgress()).toBe(false)
      })
    })

    describe('shouldCheckForUpdate', () => {
      it('should return false when autoCheck is disabled', () => {
        const { setAutoCheck } = useUpdateStore.getState()

        setAutoCheck(false)

        const { shouldCheckForUpdate } = useUpdateStore.getState()
        expect(shouldCheckForUpdate()).toBe(false)
      })

      it('should return true when autoCheck is enabled and never checked', () => {
        const { shouldCheckForUpdate } = useUpdateStore.getState()
        expect(shouldCheckForUpdate()).toBe(true)
      })

      it('should return true when check interval has passed', () => {
        const { setLastCheckTime, setCheckInterval } = useUpdateStore.getState()
        const now = Date.now()

        // Set last check to 25 hours ago
        setLastCheckTime(now - 25 * 60 * 60 * 1000)
        setCheckInterval(24)

        const { shouldCheckForUpdate } = useUpdateStore.getState()
        expect(shouldCheckForUpdate()).toBe(true)
      })

      it('should return false when check interval has not passed', () => {
        const { setLastCheckTime, setCheckInterval } = useUpdateStore.getState()
        const now = Date.now()

        // Set last check to 1 hour ago
        setLastCheckTime(now - 60 * 60 * 1000)
        setCheckInterval(24)

        const { shouldCheckForUpdate } = useUpdateStore.getState()
        expect(shouldCheckForUpdate()).toBe(false)
      })
    })
  })

  describe('resetProgress', () => {
    it('should reset progress-related state', () => {
      const { setProgress, setUpdateStatus, setError, setErrorDetails, setCanCancel, resetProgress } = useUpdateStore.getState()

      setProgress(mockUpdateProgress())
      setUpdateStatus('downloading')
      setError('Some error')
      setErrorDetails('Error details')
      setCanCancel(false)

      resetProgress()

      const state = useUpdateStore.getState()
      expect(state.progress).toBeNull()
      expect(state.updateStatus).toBe('idle')
      expect(state.error).toBeNull()
      expect(state.errorDetails).toBeNull()
      expect(state.canCancel).toBe(true)
    })
  })

  describe('reset', () => {
    it('should reset all state except settings and updateHistory', () => {
      const {
        setCurrentVersion,
        setLatestVersion,
        setUpdateInfo,
        setHasUpdate,
        setIsChecking,
        setLastCheckTime,
        setProgress,
        setUpdateStatus,
        setBackups,
        setIsCreatingBackup,
        setOfflinePackagePath,
        setError,
        setErrorDetails,
        setCanCancel,
        setNeedsRestart,
        addUpdateHistory,
        reset,
      } = useUpdateStore.getState()

      // Set various states
      setCurrentVersion('1.0.0')
      setLatestVersion('1.1.0')
      setUpdateInfo(mockUpdateInfo())
      setHasUpdate(true)
      setIsChecking(true)
      setLastCheckTime(Date.now())
      setProgress(mockUpdateProgress())
      setUpdateStatus('downloading')
      setBackups([{ createdAt: '2024-01-01', version: '1.0.0', path: '/backup' }])
      setIsCreatingBackup(true)
      setOfflinePackagePath('/path/to/package')
      setError('Error')
      setErrorDetails('Details')
      setCanCancel(false)
      setNeedsRestart(true)
      addUpdateHistory({ version: '1.0.0', timestamp: Date.now(), status: 'success' })

      // Reset
      reset()

      // Verify reset state
      const state = useUpdateStore.getState()
      expect(state.currentVersion).toBeNull()
      expect(state.latestVersion).toBeNull()
      expect(state.updateInfo).toBeNull()
      expect(state.hasUpdate).toBe(false)
      expect(state.isChecking).toBe(false)
      expect(state.lastCheckTime).toBeNull()
      expect(state.progress).toBeNull()
      expect(state.updateStatus).toBe('idle')
      expect(state.backups).toEqual([])
      expect(state.isCreatingBackup).toBe(false)
      expect(state.offlinePackagePath).toBeNull()
      expect(state.error).toBeNull()
      expect(state.errorDetails).toBeNull()
      expect(state.canCancel).toBe(true)
      expect(state.needsRestart).toBe(false)

      // Settings and history should be preserved
      expect(state.settings.autoCheck).toBe(true)
      expect(state.updateHistory).toHaveLength(1)
    })
  })

  describe('persistence', () => {
    it('should have persist middleware configured', () => {
      const state = useUpdateStore.getState()

      // Verify all expected methods exist
      expect(typeof state.setCurrentVersion).toBe('function')
      expect(typeof state.setLatestVersion).toBe('function')
      expect(typeof state.setUpdateInfo).toBe('function')
      expect(typeof state.setProgress).toBe('function')
      expect(typeof state.updateProgress).toBe('function')
      expect(typeof state.setSettings).toBe('function')
      expect(typeof state.addUpdateHistory).toBe('function')
      expect(typeof state.isUpdateAvailable).toBe('function')
      expect(typeof state.isUpdateInProgress).toBe('function')
      expect(typeof state.shouldCheckForUpdate).toBe('function')
      expect(typeof state.resetProgress).toBe('function')
      expect(typeof state.reset).toBe('function')
    })
  })
})
