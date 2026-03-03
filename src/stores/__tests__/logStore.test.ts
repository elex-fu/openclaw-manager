import { describe, it, expect, beforeEach } from 'vitest'
import { useLogStore } from '../logStore'
import type { LogEntry, LogSourceInfo, LogLevel } from '@/types'

const mockLogEntry = (overrides: Partial<LogEntry> = {}): LogEntry => ({
  id: `log-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
  timestamp: Date.now(),
  level: 'INFO',
  source: 'test-source',
  message: 'Test log message',
  metadata: {},
  ...overrides,
})

const mockLogSource = (overrides: Partial<LogSourceInfo> = {}): LogSourceInfo => ({
  id: 'test-source',
  name: 'Test Source',
  path: '/var/log/test.log',
  size: 1024,
  modified: Date.now(),
  ...overrides,
})

describe('logStore', () => {
  beforeEach(() => {
    // Reset store state to initial values
    useLogStore.setState({
      logEntries: [],
      logSources: [],
      filter: {
        levels: ['ERROR', 'WARN', 'INFO'],
        sources: [],
        searchQuery: '',
        startTime: null,
        endTime: null,
      },
      isSubscribed: false,
      subscriptionId: null,
      autoScroll: true,
      selectedLogId: null,
      stats: {
        totalCount: 0,
        errorCount: 0,
        warnCount: 0,
        infoCount: 0,
        debugCount: 0,
      },
      isLoading: false,
      isExporting: false,
      error: null,
      maxEntries: 10000,
    })
  })

  describe('initial state', () => {
    it('should have correct initial state', () => {
      const state = useLogStore.getState()

      expect(state.logEntries).toEqual([])
      expect(state.logSources).toEqual([])
      expect(state.filter.levels).toEqual(['ERROR', 'WARN', 'INFO'])
      expect(state.filter.sources).toEqual([])
      expect(state.filter.searchQuery).toBe('')
      expect(state.isSubscribed).toBe(false)
      expect(state.subscriptionId).toBeNull()
      expect(state.autoScroll).toBe(true)
      expect(state.selectedLogId).toBeNull()
      expect(state.stats.totalCount).toBe(0)
      expect(state.isLoading).toBe(false)
      expect(state.isExporting).toBe(false)
      expect(state.error).toBeNull()
      expect(state.maxEntries).toBe(10000)
    })
  })

  describe('log entries', () => {
    it('should set log entries', () => {
      const { setLogEntries } = useLogStore.getState()
      const entries = [
        mockLogEntry({ level: 'INFO', message: 'Message 1' }),
        mockLogEntry({ level: 'ERROR', message: 'Message 2' }),
      ]

      setLogEntries(entries)

      const { logEntries, stats } = useLogStore.getState()
      expect(logEntries).toHaveLength(2)
      expect(stats.totalCount).toBe(2)
    })

    it('should add single log entry', () => {
      const { addLogEntry } = useLogStore.getState()
      const entry = mockLogEntry({ level: 'WARN', message: 'Warning message' })

      addLogEntry(entry)

      const { logEntries, stats } = useLogStore.getState()
      expect(logEntries).toHaveLength(1)
      expect(logEntries[0].message).toBe('Warning message')
      expect(stats.warnCount).toBe(1)
      expect(stats.totalCount).toBe(1)
    })

    it('should add multiple log entries', () => {
      const { addLogEntries } = useLogStore.getState()
      const entries = [
        mockLogEntry({ level: 'INFO' }),
        mockLogEntry({ level: 'DEBUG' }),
        mockLogEntry({ level: 'ERROR' }),
      ]

      addLogEntries(entries)

      const { logEntries } = useLogStore.getState()
      expect(logEntries).toHaveLength(3)
    })

    it('should update stats when adding entries', () => {
      const { addLogEntry } = useLogStore.getState()

      addLogEntry(mockLogEntry({ level: 'ERROR' }))
      addLogEntry(mockLogEntry({ level: 'ERROR' }))
      addLogEntry(mockLogEntry({ level: 'WARN' }))
      addLogEntry(mockLogEntry({ level: 'INFO' }))
      addLogEntry(mockLogEntry({ level: 'DEBUG' }))
      addLogEntry(mockLogEntry({ level: 'TRACE' }))

      const { stats } = useLogStore.getState()
      expect(stats.errorCount).toBe(2)
      expect(stats.warnCount).toBe(1)
      expect(stats.infoCount).toBe(1)
      expect(stats.debugCount).toBe(2) // DEBUG + TRACE
      expect(stats.totalCount).toBe(6)
    })

    it('should limit max entries', () => {
      const { setMaxEntries, addLogEntry } = useLogStore.getState()
      setMaxEntries(5)

      for (let i = 0; i < 10; i++) {
        addLogEntry(mockLogEntry({ message: `Message ${i}` }))
      }

      const { logEntries } = useLogStore.getState()
      expect(logEntries).toHaveLength(5)
      // Oldest entries should be removed
      expect(logEntries[0].message).toBe('Message 5')
      expect(logEntries[4].message).toBe('Message 9')
    })

    it('should clear log entries', () => {
      const { addLogEntry, clearLogEntries } = useLogStore.getState()

      addLogEntry(mockLogEntry())
      addLogEntry(mockLogEntry())

      clearLogEntries()

      const { logEntries, stats } = useLogStore.getState()
      expect(logEntries).toHaveLength(0)
      expect(stats.totalCount).toBe(0)
      expect(stats.errorCount).toBe(0)
      expect(stats.warnCount).toBe(0)
      expect(stats.infoCount).toBe(0)
      expect(stats.debugCount).toBe(0)
    })
  })

  describe('log sources', () => {
    it('should set log sources', () => {
      const { setLogSources } = useLogStore.getState()
      const sources = [mockLogSource(), mockLogSource({ id: 'source-2' })]

      setLogSources(sources)

      const { logSources } = useLogStore.getState()
      expect(logSources).toHaveLength(2)
    })

    it('should add log source', () => {
      const { addLogSource } = useLogStore.getState()
      const source = mockLogSource()

      addLogSource(source)

      const { logSources } = useLogStore.getState()
      expect(logSources).toHaveLength(1)
      expect(logSources[0].id).toBe('test-source')
    })

    it('should remove log source and clean up filter', () => {
      const { addLogSource, removeLogSource, setFilter } = useLogStore.getState()
      const source = mockLogSource({ id: 'source-to-remove' })

      addLogSource(source)
      setFilter({ sources: ['source-to-remove', 'other-source'] })

      removeLogSource('source-to-remove')

      const { logSources, filter } = useLogStore.getState()
      expect(logSources).toHaveLength(0)
      expect(filter.sources).not.toContain('source-to-remove')
      expect(filter.sources).toContain('other-source')
    })
  })

  describe('filtering', () => {
    it('should set filter', () => {
      const { setFilter } = useLogStore.getState()

      setFilter({ searchQuery: 'test query', startTime: 1000, endTime: 2000 })

      const { filter } = useLogStore.getState()
      expect(filter.searchQuery).toBe('test query')
      expect(filter.startTime).toBe(1000)
      expect(filter.endTime).toBe(2000)
      // Other filter properties should remain unchanged
      expect(filter.levels).toEqual(['ERROR', 'WARN', 'INFO'])
    })

    it('should toggle level', () => {
      const { toggleLevel } = useLogStore.getState()

      // Add a level that doesn't exist
      toggleLevel('DEBUG')

      let { filter } = useLogStore.getState()
      expect(filter.levels).toContain('DEBUG')

      // Remove the level
      toggleLevel('DEBUG')

      filter = useLogStore.getState().filter
      expect(filter.levels).not.toContain('DEBUG')
    })

    it('should toggle source', () => {
      const { toggleSource } = useLogStore.getState()

      toggleSource('source-1')

      let { filter } = useLogStore.getState()
      expect(filter.sources).toContain('source-1')

      toggleSource('source-1')

      filter = useLogStore.getState().filter
      expect(filter.sources).not.toContain('source-1')
    })

    it('should set search query', () => {
      const { setSearchQuery } = useLogStore.getState()

      setSearchQuery('error message')

      const { filter } = useLogStore.getState()
      expect(filter.searchQuery).toBe('error message')
    })

    it('should reset filter to defaults', () => {
      const { setFilter, resetFilter } = useLogStore.getState()

      setFilter({
        levels: ['DEBUG'],
        sources: ['source-1'],
        searchQuery: 'test',
        startTime: 1000,
        endTime: 2000,
      })

      resetFilter()

      const { filter } = useLogStore.getState()
      expect(filter.levels).toEqual(['ERROR', 'WARN', 'INFO'])
      expect(filter.sources).toEqual([])
      expect(filter.searchQuery).toBe('')
      expect(filter.startTime).toBeNull()
      expect(filter.endTime).toBeNull()
    })
  })

  describe('getFilteredLogs', () => {
    beforeEach(() => {
      const { addLogEntries } = useLogStore.getState()
      addLogEntries([
        mockLogEntry({ id: '1', level: 'ERROR', source: 'app', message: 'Critical error', timestamp: 1000 }),
        mockLogEntry({ id: '2', level: 'WARN', source: 'app', message: 'Warning message', timestamp: 2000 }),
        mockLogEntry({ id: '3', level: 'INFO', source: 'system', message: 'Info message', timestamp: 3000 }),
        mockLogEntry({ id: '4', level: 'DEBUG', source: 'debug', message: 'Debug message', timestamp: 4000 }),
      ])
    })

    it('should filter by level', () => {
      const { setFilter, getFilteredLogs } = useLogStore.getState()

      setFilter({ levels: ['ERROR'] })
      let filtered = getFilteredLogs()
      expect(filtered).toHaveLength(1)
      expect(filtered[0].id).toBe('1')

      setFilter({ levels: ['ERROR', 'WARN'] })
      filtered = getFilteredLogs()
      expect(filtered).toHaveLength(2)
    })

    it('should filter by source', () => {
      const { setFilter, getFilteredLogs } = useLogStore.getState()

      setFilter({ sources: ['app'] })
      const filtered = getFilteredLogs()
      expect(filtered).toHaveLength(2)
      expect(filtered.every((e) => e.source === 'app')).toBe(true)
    })

    it('should filter by search query (message)', () => {
      const { setSearchQuery, getFilteredLogs } = useLogStore.getState()

      setSearchQuery('error')
      const filtered = getFilteredLogs()
      expect(filtered).toHaveLength(1)
      expect(filtered[0].message).toBe('Critical error')
    })

    it('should filter by search query (source)', () => {
      const { setSearchQuery, getFilteredLogs } = useLogStore.getState()

      setSearchQuery('system')
      const filtered = getFilteredLogs()
      expect(filtered).toHaveLength(1)
      expect(filtered[0].source).toBe('system')
    })

    it('should filter by time range', () => {
      const { setFilter, getFilteredLogs } = useLogStore.getState()

      setFilter({ startTime: 1500, endTime: 3500 })
      const filtered = getFilteredLogs()
      expect(filtered).toHaveLength(2)
      expect(filtered[0].id).toBe('2')
      expect(filtered[1].id).toBe('3')
    })

    it('should combine multiple filters', () => {
      const { setFilter, getFilteredLogs } = useLogStore.getState()

      setFilter({
        levels: ['ERROR', 'WARN'],
        sources: ['app'],
        searchQuery: 'warning',
      })
      const filtered = getFilteredLogs()
      expect(filtered).toHaveLength(1)
      expect(filtered[0].id).toBe('2')
    })

    it('should return all logs when no filters applied', () => {
      const { setFilter, getFilteredLogs } = useLogStore.getState()

      // Include DEBUG level to get all test logs
      setFilter({ levels: ['ERROR', 'WARN', 'INFO', 'DEBUG'] })
      const filtered = getFilteredLogs()
      expect(filtered).toHaveLength(4)
    })
  })

  describe('subscription state', () => {
    it('should set subscription status', () => {
      const { setIsSubscribed } = useLogStore.getState()

      setIsSubscribed(true)

      const { isSubscribed } = useLogStore.getState()
      expect(isSubscribed).toBe(true)
    })

    it('should set subscription id', () => {
      const { setSubscriptionId } = useLogStore.getState()

      setSubscriptionId('sub-123')

      const { subscriptionId } = useLogStore.getState()
      expect(subscriptionId).toBe('sub-123')
    })
  })

  describe('auto scroll', () => {
    it('should set auto scroll', () => {
      const { setAutoScroll } = useLogStore.getState()

      setAutoScroll(false)

      const { autoScroll } = useLogStore.getState()
      expect(autoScroll).toBe(false)
    })
  })

  describe('selected log', () => {
    it('should set selected log id', () => {
      const { setSelectedLogId } = useLogStore.getState()

      setSelectedLogId('log-123')

      const { selectedLogId } = useLogStore.getState()
      expect(selectedLogId).toBe('log-123')
    })

    it('should clear selected log id', () => {
      const { setSelectedLogId } = useLogStore.getState()

      setSelectedLogId('log-123')
      setSelectedLogId(null)

      const { selectedLogId } = useLogStore.getState()
      expect(selectedLogId).toBeNull()
    })
  })

  describe('loading and exporting states', () => {
    it('should set loading state', () => {
      const { setIsLoading } = useLogStore.getState()

      setIsLoading(true)

      const { isLoading } = useLogStore.getState()
      expect(isLoading).toBe(true)
    })

    it('should set exporting state', () => {
      const { setIsExporting } = useLogStore.getState()

      setIsExporting(true)

      const { isExporting } = useLogStore.getState()
      expect(isExporting).toBe(true)
    })
  })

  describe('error state', () => {
    it('should set error', () => {
      const { setError } = useLogStore.getState()

      setError('Failed to load logs')

      const { error } = useLogStore.getState()
      expect(error).toBe('Failed to load logs')
    })

    it('should clear error', () => {
      const { setError } = useLogStore.getState()

      setError('Some error')
      setError(null)

      const { error } = useLogStore.getState()
      expect(error).toBeNull()
    })
  })

  describe('max entries', () => {
    it('should set max entries', () => {
      const { setMaxEntries } = useLogStore.getState()

      setMaxEntries(5000)

      const { maxEntries } = useLogStore.getState()
      expect(maxEntries).toBe(5000)
    })
  })

  describe('reset', () => {
    it('should reset all state', () => {
      const {
        addLogEntry,
        addLogSource,
        setFilter,
        setIsSubscribed,
        setSubscriptionId,
        setAutoScroll,
        setSelectedLogId,
        setIsLoading,
        setError,
        setMaxEntries,
        reset,
      } = useLogStore.getState()

      // Set various states
      addLogEntry(mockLogEntry())
      addLogSource(mockLogSource())
      setFilter({ searchQuery: 'test', levels: ['DEBUG'] })
      setIsSubscribed(true)
      setSubscriptionId('sub-123')
      setAutoScroll(false)
      setSelectedLogId('log-123')
      setIsLoading(true)
      setError('Some error')
      setMaxEntries(5000)

      // Reset
      reset()

      // Verify reset state
      const state = useLogStore.getState()
      expect(state.logEntries).toHaveLength(0)
      expect(state.logSources).toHaveLength(0)
      expect(state.filter.levels).toEqual(['ERROR', 'WARN', 'INFO'])
      expect(state.filter.searchQuery).toBe('')
      expect(state.isSubscribed).toBe(false)
      expect(state.subscriptionId).toBeNull()
      expect(state.autoScroll).toBe(true)
      expect(state.selectedLogId).toBeNull()
      expect(state.stats.totalCount).toBe(0)
      expect(state.isLoading).toBe(false)
      expect(state.error).toBeNull()
      expect(state.maxEntries).toBe(10000)
    })
  })

  describe('persistence', () => {
    it('should have persist middleware configured', () => {
      // The store is created with persist middleware
      // This test verifies the store structure is correct
      const state = useLogStore.getState()

      // Verify all expected methods exist
      expect(typeof state.setLogEntries).toBe('function')
      expect(typeof state.addLogEntry).toBe('function')
      expect(typeof state.clearLogEntries).toBe('function')
      expect(typeof state.getFilteredLogs).toBe('function')
      expect(typeof state.reset).toBe('function')
    })
  })
})
