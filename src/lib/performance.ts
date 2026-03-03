/**
 * 性能监控工具
 * 用于跟踪应用性能指标
 */

interface PerformanceMetrics {
  // 页面加载时间
  pageLoadTime: number;
  // 首次内容绘制
  fcp: number;
  // 最大内容绘制
  lcp: number;
  // 首次输入延迟
  fid: number;
  // 累积布局偏移
  cls: number;
  // 内存使用（如果可用）
  memory?: {
    usedJSHeapSize: number;
    totalJSHeapSize: number;
    jsHeapSizeLimit: number;
  };
}

interface RenderMetrics {
  componentName: string;
  renderCount: number;
  averageRenderTime: number;
  lastRenderTime: number;
}

class PerformanceMonitor {
  private static instance: PerformanceMonitor;
  private metrics: Partial<PerformanceMetrics> = {};
  private renderMetrics: Map<string, RenderMetrics> = new Map();
  private observers: Set<(metrics: PerformanceMetrics) => void> = new Set();

  static getInstance(): PerformanceMonitor {
    if (!PerformanceMonitor.instance) {
      PerformanceMonitor.instance = new PerformanceMonitor();
    }
    return PerformanceMonitor.instance;
  }

  constructor() {
    if (typeof window !== 'undefined') {
      this.initWebVitals();
      this.initMemoryTracking();
    }
  }

  private initWebVitals() {
    // 监听页面加载完成
    window.addEventListener('load', () => {
      setTimeout(() => {
        this.measurePageLoad();
        this.measureFCP();
        this.measureLCP();
        this.measureFID();
        this.measureCLS();
      }, 0);
    });
  }

  private measurePageLoad() {
    const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
    if (navigation) {
      this.metrics.pageLoadTime = navigation.loadEventEnd - navigation.startTime;
    }
  }

  private measureFCP() {
    const observer = new PerformanceObserver((list) => {
      const entries = list.getEntries();
      const fcpEntry = entries.find(entry => entry.name === 'first-contentful-paint');
      if (fcpEntry) {
        this.metrics.fcp = fcpEntry.startTime;
      }
    });
    observer.observe({ entryTypes: ['paint'] });
  }

  private measureLCP() {
    const observer = new PerformanceObserver((list) => {
      const entries = list.getEntries();
      const lastEntry = entries[entries.length - 1];
      if (lastEntry) {
        this.metrics.lcp = lastEntry.startTime;
      }
    });
    observer.observe({ entryTypes: ['largest-contentful-paint'] });
  }

  private measureFID() {
    const observer = new PerformanceObserver((list) => {
      const entries = list.getEntries();
      const firstEntry = entries[0];
      if (firstEntry) {
        this.metrics.fid = (firstEntry as PerformanceEventTiming).processingStart - firstEntry.startTime;
      }
    });
    observer.observe({ entryTypes: ['first-input'] });
  }

  private measureCLS() {
    let clsValue = 0;
    const observer = new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        if (!(entry as any).hadRecentInput) {
          clsValue += (entry as any).value;
        }
      }
      this.metrics.cls = clsValue;
    });
    observer.observe({ entryTypes: ['layout-shift'] });
  }

  private initMemoryTracking() {
    if ('memory' in performance) {
      setInterval(() => {
        const memory = (performance as any).memory;
        if (memory) {
          this.metrics.memory = {
            usedJSHeapSize: memory.usedJSHeapSize,
            totalJSHeapSize: memory.totalJSHeapSize,
            jsHeapSizeLimit: memory.jsHeapSizeLimit,
          };
        }
      }, 5000); // 每5秒检查一次内存
    }
  }

  // 记录组件渲染性能
  recordRender(componentName: string, renderTime: number) {
    const existing = this.renderMetrics.get(componentName);
    if (existing) {
      const newCount = existing.renderCount + 1;
      const newAverage = (existing.averageRenderTime * existing.renderCount + renderTime) / newCount;
      this.renderMetrics.set(componentName, {
        componentName,
        renderCount: newCount,
        averageRenderTime: newAverage,
        lastRenderTime: renderTime,
      });
    } else {
      this.renderMetrics.set(componentName, {
        componentName,
        renderCount: 1,
        averageRenderTime: renderTime,
        lastRenderTime: renderTime,
      });
    }
  }

  // 获取所有性能指标
  getMetrics(): PerformanceMetrics {
    return {
      pageLoadTime: this.metrics.pageLoadTime || 0,
      fcp: this.metrics.fcp || 0,
      lcp: this.metrics.lcp || 0,
      fid: this.metrics.fid || 0,
      cls: this.metrics.cls || 0,
      memory: this.metrics.memory,
    };
  }

  // 获取组件渲染指标
  getRenderMetrics(): RenderMetrics[] {
    return Array.from(this.renderMetrics.values());
  }

  // 订阅性能指标更新
  subscribe(callback: (metrics: PerformanceMetrics) => void): () => void {
    this.observers.add(callback);
    return () => this.observers.delete(callback);
  }

  // 打印性能报告
  printReport() {
    console.group('📊 Performance Report');
    console.log('Page Load Time:', this.metrics.pageLoadTime?.toFixed(2), 'ms');
    console.log('FCP:', this.metrics.fcp?.toFixed(2), 'ms');
    console.log('LCP:', this.metrics.lcp?.toFixed(2), 'ms');
    console.log('FID:', this.metrics.fid?.toFixed(2), 'ms');
    console.log('CLS:', this.metrics.cls?.toFixed(4));

    if (this.metrics.memory) {
      const usedMB = this.metrics.memory.usedJSHeapSize / 1024 / 1024;
      const totalMB = this.metrics.memory.totalJSHeapSize / 1024 / 1024;
      console.log('Memory Used:', usedMB.toFixed(2), 'MB /', totalMB.toFixed(2), 'MB');
    }

    if (this.renderMetrics.size > 0) {
      console.group('Component Render Metrics');
      this.renderMetrics.forEach((metric) => {
        console.log(
          `${metric.componentName}:`,
          `${metric.renderCount} renders,`,
          `avg ${metric.averageRenderTime.toFixed(2)}ms`
        );
      });
      console.groupEnd();
    }

    console.groupEnd();
  }
}

export const performanceMonitor = PerformanceMonitor.getInstance();

// React 性能追踪 Hook
export function usePerformanceTrack(componentName: string) {
  const startTime = performance.now();

  return {
    trackRender: () => {
      const endTime = performance.now();
      performanceMonitor.recordRender(componentName, endTime - startTime);
    },
  };
}

// 防抖函数
export function debounce<T extends (...args: unknown[]) => unknown>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout> | null = null;

  return (...args: Parameters<T>) => {
    if (timeout) clearTimeout(timeout);
    timeout = setTimeout(() => func(...args), wait);
  };
}

// 节流函数
export function throttle<T extends (...args: unknown[]) => unknown>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle = false;

  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      func(...args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}

// 测量函数执行时间
export function measurePerformance<T>(
  name: string,
  fn: () => T
): T {
  const start = performance.now();
  const result = fn();
  const end = performance.now();
  console.log(`⏱️ ${name}: ${(end - start).toFixed(2)}ms`);
  return result;
}

// 异步测量
export async function measurePerformanceAsync<T>(
  name: string,
  fn: () => Promise<T>
): Promise<T> {
  const start = performance.now();
  const result = await fn();
  const end = performance.now();
  console.log(`⏱️ ${name}: ${(end - start).toFixed(2)}ms`);
  return result;
}
