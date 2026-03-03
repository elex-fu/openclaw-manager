import { describe, it, expect, beforeEach } from 'vitest'
import { useSkillStore } from '../skillStore'
import type { Skill, InstalledSkill, SkillCategory, SkillMarketItem } from '@/types'

const mockSkill = (overrides: Partial<Skill> = {}): Skill => ({
  id: 'skill-1',
  name: 'Test Skill',
  description: 'A test skill',
  author: 'Test Author',
  version: '1.0.0',
  categories: ['productivity'],
  tags: ['test', 'demo'],
  rating: 4.5,
  downloads: 100,
  hooks: [],
  dependencies: [],
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-15T00:00:00Z',
  ...overrides,
})

const mockInstalledSkill = (overrides: Partial<InstalledSkill> = {}): InstalledSkill => ({
  ...mockSkill(),
  is_enabled: true,
  config: {},
  installed_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-15T00:00:00Z',
  has_update: false,
  ...overrides,
})

const mockCategory = (overrides: Partial<SkillCategory> = {}): SkillCategory => ({
  id: 'productivity',
  name: 'Productivity',
  description: 'Productivity tools',
  icon: 'zap',
  sort_order: 1,
  ...overrides,
})

const mockMarketItem = (overrides: Partial<SkillMarketItem> = {}): SkillMarketItem => ({
  id: 'skill-1',
  name: 'Test Skill',
  description: 'A test skill',
  author: 'Test Author',
  version: '1.0.0',
  categories: ['productivity'],
  tags: ['test', 'demo'],
  rating: 4.5,
  downloads: 100,
  is_installed: false,
  is_enabled: false,
  ...overrides,
})

describe('skillStore', () => {
  beforeEach(() => {
    // Reset store state to initial values
    useSkillStore.setState({
      marketSkills: [],
      installedSkills: [],
      categories: [],
      marketItems: [],
      searchQuery: '',
      selectedCategory: null,
      selectedTags: [],
      sortBy: 'relevance',
      sortOrder: 'desc',
      currentPage: 1,
      totalPages: 1,
      itemsPerPage: 20,
      selectedSkillId: null,
      isLoading: false,
      isInstalling: false,
      installingSkillId: null,
      error: null,
    })
  })

  describe('initial state', () => {
    it('should have correct initial state', () => {
      const state = useSkillStore.getState()

      expect(state.marketSkills).toEqual([])
      expect(state.installedSkills).toEqual([])
      expect(state.categories).toEqual([])
      expect(state.marketItems).toEqual([])
      expect(state.searchQuery).toBe('')
      expect(state.selectedCategory).toBeNull()
      expect(state.selectedTags).toEqual([])
      expect(state.sortBy).toBe('relevance')
      expect(state.sortOrder).toBe('desc')
      expect(state.currentPage).toBe(1)
      expect(state.totalPages).toBe(1)
      expect(state.itemsPerPage).toBe(20)
      expect(state.selectedSkillId).toBeNull()
      expect(state.isLoading).toBe(false)
      expect(state.isInstalling).toBe(false)
      expect(state.installingSkillId).toBeNull()
      expect(state.error).toBeNull()
    })
  })

  describe('market skills', () => {
    it('should set market skills', () => {
      const { setMarketSkills } = useSkillStore.getState()
      const skills = [mockSkill({ id: '1' }), mockSkill({ id: '2' })]

      setMarketSkills(skills)

      const { marketSkills } = useSkillStore.getState()
      expect(marketSkills).toHaveLength(2)
      expect(marketSkills[0].id).toBe('1')
      expect(marketSkills[1].id).toBe('2')
    })

    it('should add market skills without duplicates', () => {
      const { setMarketSkills, addMarketSkills } = useSkillStore.getState()
      const initialSkills = [mockSkill({ id: '1' })]
      const newSkills = [mockSkill({ id: '1' }), mockSkill({ id: '2' })]

      setMarketSkills(initialSkills)
      addMarketSkills(newSkills)

      const { marketSkills } = useSkillStore.getState()
      expect(marketSkills).toHaveLength(2)
    })

    it('should add only new market skills', () => {
      const { setMarketSkills, addMarketSkills } = useSkillStore.getState()
      const initialSkills = [mockSkill({ id: '1' })]
      const newSkills = [mockSkill({ id: '2' }), mockSkill({ id: '3' })]

      setMarketSkills(initialSkills)
      addMarketSkills(newSkills)

      const { marketSkills } = useSkillStore.getState()
      expect(marketSkills).toHaveLength(3)
    })
  })

  describe('installed skills', () => {
    it('should set installed skills', () => {
      const { setInstalledSkills } = useSkillStore.getState()
      const skills = [mockInstalledSkill({ id: '1' }), mockInstalledSkill({ id: '2' })]

      setInstalledSkills(skills)

      const { installedSkills } = useSkillStore.getState()
      expect(installedSkills).toHaveLength(2)
    })

    it('should add installed skill', () => {
      const { addInstalledSkill } = useSkillStore.getState()
      const skill = mockInstalledSkill({ id: '1' })

      addInstalledSkill(skill)

      const { installedSkills } = useSkillStore.getState()
      expect(installedSkills).toHaveLength(1)
      expect(installedSkills[0].id).toBe('1')
    })

    it('should update installed skill', () => {
      const { addInstalledSkill, updateInstalledSkill } = useSkillStore.getState()
      const skill = mockInstalledSkill({ id: '1', name: 'Old Name' })

      addInstalledSkill(skill)
      updateInstalledSkill('1', { name: 'New Name', is_enabled: false })

      const { installedSkills } = useSkillStore.getState()
      expect(installedSkills[0].name).toBe('New Name')
      expect(installedSkills[0].is_enabled).toBe(false)
    })

    it('should remove installed skill', () => {
      const { addInstalledSkill, removeInstalledSkill } = useSkillStore.getState()
      const skill = mockInstalledSkill({ id: '1' })

      addInstalledSkill(skill)
      removeInstalledSkill('1')

      const { installedSkills } = useSkillStore.getState()
      expect(installedSkills).toHaveLength(0)
    })
  })

  describe('categories', () => {
    it('should set categories', () => {
      const { setCategories } = useSkillStore.getState()
      const categories = [mockCategory(), mockCategory({ id: 'utility', name: 'Utility' })]

      setCategories(categories)

      const { categories: storedCategories } = useSkillStore.getState()
      expect(storedCategories).toHaveLength(2)
    })
  })

  describe('market items', () => {
    it('should set market items', () => {
      const { setMarketItems } = useSkillStore.getState()
      const items = [mockMarketItem({ id: '1' }), mockMarketItem({ id: '2' })]

      setMarketItems(items)

      const { marketItems } = useSkillStore.getState()
      expect(marketItems).toHaveLength(2)
    })

    it('should update market item', () => {
      const { setMarketItems, updateMarketItem } = useSkillStore.getState()
      const items = [mockMarketItem({ id: '1', is_installed: false })]

      setMarketItems(items)
      updateMarketItem('1', { is_installed: true, is_enabled: true })

      const { marketItems } = useSkillStore.getState()
      expect(marketItems[0].is_installed).toBe(true)
      expect(marketItems[0].is_enabled).toBe(true)
    })
  })

  describe('search', () => {
    it('should set search query and reset page', () => {
      const { setSearchQuery, setCurrentPage } = useSkillStore.getState()

      setCurrentPage(5)
      setSearchQuery('search term')

      const { searchQuery, currentPage } = useSkillStore.getState()
      expect(searchQuery).toBe('search term')
      expect(currentPage).toBe(1)
    })
  })

  describe('category filter', () => {
    it('should set selected category and reset page', () => {
      const { setSelectedCategory, setCurrentPage } = useSkillStore.getState()

      setCurrentPage(3)
      setSelectedCategory('productivity')

      const { selectedCategory, currentPage } = useSkillStore.getState()
      expect(selectedCategory).toBe('productivity')
      expect(currentPage).toBe(1)
    })

    it('should clear selected category', () => {
      const { setSelectedCategory } = useSkillStore.getState()

      setSelectedCategory('productivity')
      setSelectedCategory(null)

      const { selectedCategory } = useSkillStore.getState()
      expect(selectedCategory).toBeNull()
    })
  })

  describe('tag filter', () => {
    it('should set selected tags and reset page', () => {
      const { setSelectedTags, setCurrentPage } = useSkillStore.getState()

      setCurrentPage(3)
      setSelectedTags(['tag1', 'tag2'])

      const { selectedTags, currentPage } = useSkillStore.getState()
      expect(selectedTags).toEqual(['tag1', 'tag2'])
      expect(currentPage).toBe(1)
    })

    it('should toggle tag on', () => {
      const { toggleTag } = useSkillStore.getState()

      toggleTag('tag1')

      const { selectedTags } = useSkillStore.getState()
      expect(selectedTags).toContain('tag1')
    })

    it('should toggle tag off', () => {
      const { setSelectedTags, toggleTag } = useSkillStore.getState()

      setSelectedTags(['tag1', 'tag2'])
      toggleTag('tag1')

      const { selectedTags } = useSkillStore.getState()
      expect(selectedTags).not.toContain('tag1')
      expect(selectedTags).toContain('tag2')
    })
  })

  describe('sorting', () => {
    it('should set sort by', () => {
      const { setSortBy } = useSkillStore.getState()

      setSortBy('downloads')

      const { sortBy } = useSkillStore.getState()
      expect(sortBy).toBe('downloads')
    })

    it('should set sort order', () => {
      const { setSortOrder } = useSkillStore.getState()

      setSortOrder('asc')

      const { sortOrder } = useSkillStore.getState()
      expect(sortOrder).toBe('asc')
    })
  })

  describe('pagination', () => {
    it('should set current page', () => {
      const { setCurrentPage } = useSkillStore.getState()

      setCurrentPage(5)

      const { currentPage } = useSkillStore.getState()
      expect(currentPage).toBe(5)
    })

    it('should set total pages', () => {
      const { setTotalPages } = useSkillStore.getState()

      setTotalPages(10)

      const { totalPages } = useSkillStore.getState()
      expect(totalPages).toBe(10)
    })

    it('should set items per page and reset page', () => {
      const { setItemsPerPage, setCurrentPage } = useSkillStore.getState()

      setCurrentPage(5)
      setItemsPerPage(50)

      const { itemsPerPage, currentPage } = useSkillStore.getState()
      expect(itemsPerPage).toBe(50)
      expect(currentPage).toBe(1)
    })
  })

  describe('selected skill', () => {
    it('should set selected skill id', () => {
      const { setSelectedSkillId } = useSkillStore.getState()

      setSelectedSkillId('skill-123')

      const { selectedSkillId } = useSkillStore.getState()
      expect(selectedSkillId).toBe('skill-123')
    })

    it('should clear selected skill id', () => {
      const { setSelectedSkillId } = useSkillStore.getState()

      setSelectedSkillId('skill-123')
      setSelectedSkillId(null)

      const { selectedSkillId } = useSkillStore.getState()
      expect(selectedSkillId).toBeNull()
    })
  })

  describe('loading states', () => {
    it('should set loading state', () => {
      const { setIsLoading } = useSkillStore.getState()

      setIsLoading(true)

      const { isLoading } = useSkillStore.getState()
      expect(isLoading).toBe(true)
    })

    it('should set installing state', () => {
      const { setIsInstalling } = useSkillStore.getState()

      setIsInstalling(true)

      const { isInstalling } = useSkillStore.getState()
      expect(isInstalling).toBe(true)
    })

    it('should set installing skill id', () => {
      const { setInstallingSkillId } = useSkillStore.getState()

      setInstallingSkillId('skill-123')

      const { installingSkillId } = useSkillStore.getState()
      expect(installingSkillId).toBe('skill-123')
    })
  })

  describe('error state', () => {
    it('should set error', () => {
      const { setError } = useSkillStore.getState()

      setError('Failed to load skills')

      const { error } = useSkillStore.getState()
      expect(error).toBe('Failed to load skills')
    })

    it('should clear error', () => {
      const { setError } = useSkillStore.getState()

      setError('Some error')
      setError(null)

      const { error } = useSkillStore.getState()
      expect(error).toBeNull()
    })
  })

  describe('getFilteredMarketSkills', () => {
    beforeEach(() => {
      const { setMarketSkills } = useSkillStore.getState()
      setMarketSkills([
        mockSkill({ id: '1', name: 'Alpha Skill', categories: ['productivity'], tags: ['test'], downloads: 100, rating: 4.0, created_at: '2024-01-01T00:00:00Z', updated_at: '2024-01-01T00:00:00Z' }),
        mockSkill({ id: '2', name: 'Beta Tool', categories: ['utility'], tags: ['demo'], downloads: 200, rating: 4.5, created_at: '2024-02-01T00:00:00Z', updated_at: '2024-02-01T00:00:00Z' }),
        mockSkill({ id: '3', name: 'Gamma App', categories: ['productivity'], tags: ['test', 'demo'], downloads: 50, rating: 3.5, created_at: '2024-03-01T00:00:00Z', updated_at: '2024-03-01T00:00:00Z' }),
      ])
    })

    it('should filter by search query (name)', () => {
      const { setSearchQuery, getFilteredMarketSkills } = useSkillStore.getState()

      setSearchQuery('alpha')
      const filtered = getFilteredMarketSkills()
      expect(filtered).toHaveLength(1)
      expect(filtered[0].id).toBe('1')
    })

    it('should filter by search query (description)', () => {
      const { setSearchQuery, getFilteredMarketSkills } = useSkillStore.getState()

      setSearchQuery('test skill')
      const filtered = getFilteredMarketSkills()
      expect(filtered.length).toBeGreaterThan(0)
    })

    it('should filter by search query (tags)', () => {
      const { setSearchQuery, getFilteredMarketSkills } = useSkillStore.getState()

      setSearchQuery('demo')
      const filtered = getFilteredMarketSkills()
      expect(filtered).toHaveLength(2)
    })

    it('should filter by category', () => {
      const { setSelectedCategory, getFilteredMarketSkills } = useSkillStore.getState()

      setSelectedCategory('productivity')
      const filtered = getFilteredMarketSkills()
      expect(filtered).toHaveLength(2)
      expect(filtered.every(s => s.categories.includes('productivity'))).toBe(true)
    })

    it('should filter by tags', () => {
      const { setSelectedTags, getFilteredMarketSkills } = useSkillStore.getState()

      setSelectedTags(['test'])
      const filtered = getFilteredMarketSkills()
      expect(filtered).toHaveLength(2)
    })

    it('should sort by downloads', () => {
      const { setSortBy, getFilteredMarketSkills } = useSkillStore.getState()

      setSortBy('downloads')
      const filtered = getFilteredMarketSkills()
      expect(filtered[0].id).toBe('2') // 200 downloads
      expect(filtered[1].id).toBe('1') // 100 downloads
      expect(filtered[2].id).toBe('3') // 50 downloads
    })

    it('should sort by rating', () => {
      const { setSortBy, getFilteredMarketSkills } = useSkillStore.getState()

      setSortBy('rating')
      const filtered = getFilteredMarketSkills()
      expect(filtered[0].id).toBe('2') // 4.5 rating
      expect(filtered[1].id).toBe('1') // 4.0 rating
      expect(filtered[2].id).toBe('3') // 3.5 rating
    })

    it('should sort by created_at', () => {
      const { setSortBy, getFilteredMarketSkills } = useSkillStore.getState()

      setSortBy('created_at')
      const filtered = getFilteredMarketSkills()
      expect(filtered[0].id).toBe('3') // March
      expect(filtered[1].id).toBe('2') // February
      expect(filtered[2].id).toBe('1') // January
    })

    it('should sort in ascending order', () => {
      const { setSortBy, setSortOrder, getFilteredMarketSkills } = useSkillStore.getState()

      setSortBy('downloads')
      setSortOrder('asc')
      const filtered = getFilteredMarketSkills()
      expect(filtered[0].id).toBe('3') // 50 downloads
      expect(filtered[1].id).toBe('1') // 100 downloads
      expect(filtered[2].id).toBe('2') // 200 downloads
    })

    it('should combine search and category filter', () => {
      const { setSearchQuery, setSelectedCategory, getFilteredMarketSkills } = useSkillStore.getState()

      setSelectedCategory('productivity')
      setSearchQuery('gamma')
      const filtered = getFilteredMarketSkills()
      expect(filtered).toHaveLength(1)
      expect(filtered[0].id).toBe('3')
    })
  })

  describe('getFilteredInstalledSkills', () => {
    beforeEach(() => {
      const { setInstalledSkills } = useSkillStore.getState()
      setInstalledSkills([
        mockInstalledSkill({ id: '1', name: 'Installed Alpha', categories: ['productivity'] }),
        mockInstalledSkill({ id: '2', name: 'Installed Beta', categories: ['utility'] }),
      ])
    })

    it('should filter installed skills by search query', () => {
      const { setSearchQuery, getFilteredInstalledSkills } = useSkillStore.getState()

      setSearchQuery('alpha')
      const filtered = getFilteredInstalledSkills()
      expect(filtered).toHaveLength(1)
      expect(filtered[0].id).toBe('1')
    })

    it('should filter installed skills by category', () => {
      const { setSelectedCategory, getFilteredInstalledSkills } = useSkillStore.getState()

      setSelectedCategory('utility')
      const filtered = getFilteredInstalledSkills()
      expect(filtered).toHaveLength(1)
      expect(filtered[0].id).toBe('2')
    })
  })

  describe('resetFilters', () => {
    it('should reset filters to defaults', () => {
      const {
        setSearchQuery,
        setSelectedCategory,
        setSelectedTags,
        setSortBy,
        setSortOrder,
        setCurrentPage,
        resetFilters,
      } = useSkillStore.getState()

      setSearchQuery('test')
      setSelectedCategory('productivity')
      setSelectedTags(['tag1'])
      setSortBy('downloads')
      setSortOrder('asc')
      setCurrentPage(5)

      resetFilters()

      const state = useSkillStore.getState()
      expect(state.searchQuery).toBe('')
      expect(state.selectedCategory).toBeNull()
      expect(state.selectedTags).toEqual([])
      expect(state.sortBy).toBe('relevance')
      expect(state.sortOrder).toBe('desc')
      expect(state.currentPage).toBe(1)
    })
  })

  describe('reset', () => {
    it('should reset all state', () => {
      const {
        setMarketSkills,
        setInstalledSkills,
        setCategories,
        setSearchQuery,
        setSelectedCategory,
        setSelectedTags,
        setSortBy,
        setCurrentPage,
        setSelectedSkillId,
        setIsLoading,
        setError,
        reset,
      } = useSkillStore.getState()

      // Set various states
      setMarketSkills([mockSkill()])
      setInstalledSkills([mockInstalledSkill()])
      setCategories([mockCategory()])
      setSearchQuery('test')
      setSelectedCategory('productivity')
      setSelectedTags(['tag1'])
      setSortBy('downloads')
      setCurrentPage(5)
      setSelectedSkillId('skill-123')
      setIsLoading(true)
      setError('Some error')

      // Reset
      reset()

      // Verify reset state
      const state = useSkillStore.getState()
      expect(state.marketSkills).toHaveLength(0)
      expect(state.installedSkills).toHaveLength(0)
      expect(state.categories).toHaveLength(0)
      expect(state.searchQuery).toBe('')
      expect(state.selectedCategory).toBeNull()
      expect(state.selectedTags).toEqual([])
      expect(state.sortBy).toBe('relevance')
      expect(state.sortOrder).toBe('desc')
      expect(state.currentPage).toBe(1)
      expect(state.totalPages).toBe(1)
      expect(state.selectedSkillId).toBeNull()
      expect(state.isLoading).toBe(false)
      expect(state.isInstalling).toBe(false)
      expect(state.error).toBeNull()
    })
  })

  describe('persistence', () => {
    it('should have persist middleware configured', () => {
      const state = useSkillStore.getState()

      // Verify all expected methods exist
      expect(typeof state.setMarketSkills).toBe('function')
      expect(typeof state.setInstalledSkills).toBe('function')
      expect(typeof state.getFilteredMarketSkills).toBe('function')
      expect(typeof state.getFilteredInstalledSkills).toBe('function')
      expect(typeof state.reset).toBe('function')
      expect(typeof state.resetFilters).toBe('function')
    })
  })
})
