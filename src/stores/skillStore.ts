import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { Skill, InstalledSkill, SkillCategory, SkillMarketItem } from '@/types';

interface SkillState {
  // 技能列表（市场）
  marketSkills: Skill[];
  setMarketSkills: (skills: Skill[]) => void;
  addMarketSkills: (skills: Skill[]) => void;

  // 已安装技能
  installedSkills: InstalledSkill[];
  setInstalledSkills: (skills: InstalledSkill[]) => void;
  addInstalledSkill: (skill: InstalledSkill) => void;
  updateInstalledSkill: (id: string, updates: Partial<InstalledSkill>) => void;
  removeInstalledSkill: (id: string) => void;

  // 技能分类
  categories: SkillCategory[];
  setCategories: (categories: SkillCategory[]) => void;

  // 技能市场项目（带安装状态）
  marketItems: SkillMarketItem[];
  setMarketItems: (items: SkillMarketItem[]) => void;
  updateMarketItem: (id: string, updates: Partial<SkillMarketItem>) => void;

  // 搜索和筛选
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  selectedCategory: string | null;
  setSelectedCategory: (category: string | null) => void;
  selectedTags: string[];
  setSelectedTags: (tags: string[]) => void;
  toggleTag: (tag: string) => void;

  // 排序
  sortBy: 'relevance' | 'downloads' | 'rating' | 'created_at' | 'updated_at';
  setSortBy: (sortBy: SkillState['sortBy']) => void;
  sortOrder: 'asc' | 'desc';
  setSortOrder: (order: 'asc' | 'desc') => void;

  // 分页
  currentPage: number;
  setCurrentPage: (page: number) => void;
  totalPages: number;
  setTotalPages: (pages: number) => void;
  itemsPerPage: number;
  setItemsPerPage: (count: number) => void;

  // 选中状态
  selectedSkillId: string | null;
  setSelectedSkillId: (id: string | null) => void;

  // 加载状态
  isLoading: boolean;
  setIsLoading: (loading: boolean) => void;
  isInstalling: boolean;
  setIsInstalling: (installing: boolean) => void;
  installingSkillId: string | null;
  setInstallingSkillId: (id: string | null) => void;

  // 错误状态
  error: string | null;
  setError: (error: string | null) => void;

  // 筛选后的技能列表（计算属性）
  getFilteredMarketSkills: () => Skill[];
  getFilteredInstalledSkills: () => InstalledSkill[];

  // 重置
  reset: () => void;
  resetFilters: () => void;
}

const defaultItemsPerPage = 20;

export const useSkillStore = create<SkillState>()(
  persist(
    (set, get) => ({
      // 初始状态
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
      itemsPerPage: defaultItemsPerPage,
      selectedSkillId: null,
      isLoading: false,
      isInstalling: false,
      installingSkillId: null,
      error: null,

      // 市场技能管理
      setMarketSkills: (skills) => set({ marketSkills: skills }),
      addMarketSkills: (skills) =>
        set((state) => {
          const existingIds = new Set(state.marketSkills.map((s) => s.id));
          const newSkills = skills.filter((s) => !existingIds.has(s.id));
          return { marketSkills: [...state.marketSkills, ...newSkills] };
        }),

      // 已安装技能管理
      setInstalledSkills: (skills) => set({ installedSkills: skills }),
      addInstalledSkill: (skill) =>
        set((state) => ({
          installedSkills: [...state.installedSkills, skill],
        })),
      updateInstalledSkill: (id, updates) =>
        set((state) => ({
          installedSkills: state.installedSkills.map((s) =>
            s.id === id ? { ...s, ...updates } : s
          ),
        })),
      removeInstalledSkill: (id) =>
        set((state) => ({
          installedSkills: state.installedSkills.filter((s) => s.id !== id),
        })),

      // 分类管理
      setCategories: (categories) => set({ categories }),

      // 市场项目管理
      setMarketItems: (items) => set({ marketItems: items }),
      updateMarketItem: (id, updates) =>
        set((state) => ({
          marketItems: state.marketItems.map((item) =>
            item.id === id ? { ...item, ...updates } : item
          ),
        })),

      // 搜索和筛选
      setSearchQuery: (query) => set({ searchQuery: query, currentPage: 1 }),
      setSelectedCategory: (category) =>
        set({ selectedCategory: category, currentPage: 1 }),
      setSelectedTags: (tags) => set({ selectedTags: tags, currentPage: 1 }),
      toggleTag: (tag) =>
        set((state) => {
          const tags = state.selectedTags.includes(tag)
            ? state.selectedTags.filter((t) => t !== tag)
            : [...state.selectedTags, tag];
          return { selectedTags: tags, currentPage: 1 };
        }),

      // 排序
      setSortBy: (sortBy) => set({ sortBy }),
      setSortOrder: (sortOrder) => set({ sortOrder }),

      // 分页
      setCurrentPage: (page) => set({ currentPage: page }),
      setTotalPages: (pages) => set({ totalPages: pages }),
      setItemsPerPage: (count) => set({ itemsPerPage: count, currentPage: 1 }),

      // 选中状态
      setSelectedSkillId: (id) => set({ selectedSkillId: id }),

      // 加载状态
      setIsLoading: (isLoading) => set({ isLoading }),
      setIsInstalling: (isInstalling) => set({ isInstalling }),
      setInstallingSkillId: (id) => set({ installingSkillId: id }),

      // 错误状态
      setError: (error) => set({ error }),

      // 筛选后的市场技能
      getFilteredMarketSkills: () => {
        const state = get();
        let filtered = [...state.marketSkills];

        // 搜索筛选
        if (state.searchQuery) {
          const query = state.searchQuery.toLowerCase();
          filtered = filtered.filter(
            (s) =>
              s.name.toLowerCase().includes(query) ||
              s.description.toLowerCase().includes(query) ||
              s.tags.some((t) => t.toLowerCase().includes(query))
          );
        }

        // 分类筛选
        if (state.selectedCategory) {
          filtered = filtered.filter((s) =>
            s.categories.includes(state.selectedCategory!)
          );
        }

        // 标签筛选
        if (state.selectedTags.length > 0) {
          filtered = filtered.filter((s) =>
            state.selectedTags.some((tag) => s.tags.includes(tag))
          );
        }

        // 排序
        filtered.sort((a, b) => {
          let comparison = 0;
          switch (state.sortBy) {
            case 'downloads':
              comparison = a.downloads - b.downloads;
              break;
            case 'rating':
              comparison = a.rating - b.rating;
              break;
            case 'created_at':
              comparison =
                new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
              break;
            case 'updated_at':
              comparison =
                new Date(a.updated_at).getTime() - new Date(b.updated_at).getTime();
              break;
            default:
              comparison = 0;
          }
          return state.sortOrder === 'asc' ? comparison : -comparison;
        });

        return filtered;
      },

      // 筛选后的已安装技能
      getFilteredInstalledSkills: () => {
        const state = get();
        let filtered = [...state.installedSkills];

        // 搜索筛选
        if (state.searchQuery) {
          const query = state.searchQuery.toLowerCase();
          filtered = filtered.filter(
            (s) =>
              s.name.toLowerCase().includes(query) ||
              s.description.toLowerCase().includes(query)
          );
        }

        // 分类筛选
        if (state.selectedCategory) {
          filtered = filtered.filter((s) =>
            s.categories.includes(state.selectedCategory!)
          );
        }

        return filtered;
      },

      // 重置筛选
      resetFilters: () =>
        set({
          searchQuery: '',
          selectedCategory: null,
          selectedTags: [],
          sortBy: 'relevance',
          sortOrder: 'desc',
          currentPage: 1,
        }),

      // 重置所有状态
      reset: () =>
        set({
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
          itemsPerPage: defaultItemsPerPage,
          selectedSkillId: null,
          isLoading: false,
          isInstalling: false,
          installingSkillId: null,
          error: null,
        }),
    }),
    {
      name: 'skill-storage',
      partialize: (state) => ({
        // 只持久化用户偏好设置
        itemsPerPage: state.itemsPerPage,
        sortBy: state.sortBy,
        sortOrder: state.sortOrder,
      }),
    }
  )
);

// 选择器 - 优化订阅性能
export const selectMarketSkills = (state: SkillState) => state.marketSkills;
export const selectInstalledSkills = (state: SkillState) => state.installedSkills;
export const selectCategories = (state: SkillState) => state.categories;
export const selectSearchQuery = (state: SkillState) => state.searchQuery;
export const selectSelectedCategory = (state: SkillState) => state.selectedCategory;
export const selectIsLoading = (state: SkillState) => state.isLoading;
export const selectSelectedSkillId = (state: SkillState) => state.selectedSkillId;
