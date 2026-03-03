import React from 'react'
import { render as rtlRender } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { HashRouter } from 'react-router-dom'
import type { RenderOptions } from '@testing-library/react'

/**
 * 创建测试用的 QueryClient
 */
export function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
        staleTime: 0,
      },
    },
  })
}

/**
 * 包装组件以提供必要上下文
 */
export function TestWrapper({ children }: { children: React.ReactNode }) {
  const queryClient = createTestQueryClient()

  return (
    <QueryClientProvider client={queryClient}>
      <HashRouter>{children}</HashRouter>
    </QueryClientProvider>
  )
}

/**
 * 自定义 render 函数，包含必要的 Provider
 */
export function render(
  ui: React.ReactElement,
  options: Omit<RenderOptions, 'wrapper'> = {}
) {
  return rtlRender(ui, { wrapper: TestWrapper, ...options })
}

/**
 * 等待指定时间
 */
export function wait(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

/**
 * 创建 mock 的 API 响应
 */
export function createMockApiResponse<T>(data: T) {
  return {
    success: true,
    data,
    error: null,
  }
}

/**
 * 创建 mock 的 API 错误响应
 */
export function createMockApiError(message: string) {
  return {
    success: false,
    data: null,
    error: {
      title: '错误',
      description: message,
      action: null,
      severity: 'error',
      retryable: false,
    },
  }
}
