import { Component, type ErrorInfo, type ReactNode } from 'react'
import { AlertCircle, RefreshCw, Home, WifiOff, ServerOff, Bug } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { networkManager } from '@/lib/tauri-api'

interface Props {
  children: ReactNode
  fallback?: ReactNode
  onReset?: () => void
  onError?: (error: Error, errorInfo: ErrorInfo) => void
}

interface State {
  hasError: boolean
  error: Error | null
  errorInfo: ErrorInfo | null
  isOnline: boolean
  errorType: 'network' | 'runtime' | 'unknown'
}

// 错误分类
function classifyError(error: Error): State['errorType'] {
  const message = error.message.toLowerCase()
  if (
    message.includes('network') ||
    message.includes('fetch') ||
    message.includes('timeout') ||
    message.includes('econnrefused') ||
    message.includes('offline')
  ) {
    return 'network'
  }
  if (
    message.includes('runtime') ||
    message.includes('undefined') ||
    message.includes('null') ||
    message.includes('cannot read') ||
    message.includes('is not a function')
  ) {
    return 'runtime'
  }
  return 'unknown'
}

// 错误上报服务
class ErrorReporter {
  private static instance: ErrorReporter
  private errors: Array<{ error: Error; timestamp: number; context?: string }> = []
  private maxErrors = 50

  static getInstance(): ErrorReporter {
    if (!ErrorReporter.instance) {
      ErrorReporter.instance = new ErrorReporter()
    }
    return ErrorReporter.instance
  }

  report(error: Error, context?: string) {
    console.error('[ErrorReporter]', error, context)

    // 存储错误（内存中，限制数量）
    this.errors.push({
      error,
      timestamp: Date.now(),
      context,
    })

    if (this.errors.length > this.maxErrors) {
      this.errors.shift()
    }

    // 可以在这里添加发送到远程服务器的逻辑
    // 例如：Sentry, LogRocket 等
    if (process.env.NODE_ENV === 'production') {
      // TODO: 集成错误上报服务
      // this.sendToRemote(error, context)
    }
  }

  getRecentErrors() {
    return [...this.errors]
  }

  clear() {
    this.errors = []
  }
}

export const errorReporter = ErrorReporter.getInstance()

export class ErrorBoundary extends Component<Props, State> {
  private unsubscribeNetwork?: () => void

  public state: State = {
    hasError: false,
    error: null,
    errorInfo: null,
    isOnline: true,
    errorType: 'unknown',
  }

  componentDidMount() {
    // 监听网络状态
    this.unsubscribeNetwork = networkManager.subscribe((status) => {
      this.setState({ isOnline: status === 'online' })
    })
    this.setState({ isOnline: networkManager.isOnline() })
  }

  componentWillUnmount() {
    this.unsubscribeNetwork?.()
  }

  public static getDerivedStateFromError(error: Error): Partial<State> {
    return {
      hasError: true,
      error,
      errorType: classifyError(error),
    }
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Uncaught error:', error, errorInfo)

    this.setState({ error, errorInfo })

    // 上报错误
    errorReporter.report(error, errorInfo.componentStack ?? undefined)

    // 调用外部错误处理
    this.props.onError?.(error, errorInfo)
  }

  private handleReset = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
      errorType: 'unknown',
    })
    this.props.onReset?.()
  }

  private handleReload = () => {
    window.location.reload()
  }

  private handleGoHome = () => {
    window.location.hash = '/'
    this.handleReset()
  }

  private getErrorIcon() {
    switch (this.state.errorType) {
      case 'network':
        return this.state.isOnline ? <ServerOff className="h-8 w-8" /> : <WifiOff className="h-8 w-8" />
      case 'runtime':
        return <Bug className="h-8 w-8" />
      default:
        return <AlertCircle className="h-8 w-8" />
    }
  }

  private getErrorTitle() {
    switch (this.state.errorType) {
      case 'network':
        return this.state.isOnline ? '服务连接失败' : '网络连接已断开'
      case 'runtime':
        return '程序运行错误'
      default:
        return '出错了'
    }
  }

  private getErrorDescription() {
    switch (this.state.errorType) {
      case 'network':
        return this.state.isOnline
          ? '无法连接到后端服务，请检查服务是否正常运行'
          : '您的设备已离线，请检查网络连接后重试'
      case 'runtime':
        return '应用程序遇到了运行时错误，建议刷新页面重试'
      default:
        return '应用程序遇到了意外错误'
    }
  }

  public render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback
      }

      return (
        <div className="flex min-h-[60vh] items-center justify-center p-6">
          <Card className="w-full max-w-lg">
            <CardHeader className="text-center">
              <div className={`
                mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full
                ${this.state.errorType === 'network' ? 'bg-orange-500/10 text-orange-500' : ''}
                ${this.state.errorType === 'runtime' ? 'bg-yellow-500/10 text-yellow-500' : ''}
                ${this.state.errorType === 'unknown' ? 'bg-destructive/10 text-destructive' : ''}
              `}>
                {this.getErrorIcon()}
              </div>
              <CardTitle className="text-xl">{this.getErrorTitle()}</CardTitle>
              <CardDescription>
                {this.getErrorDescription()}
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {this.state.error && (
                <div className="rounded-lg bg-muted p-4">
                  <p className="font-mono text-sm text-destructive">
                    {this.state.error.message}
                  </p>
                  {process.env.NODE_ENV === 'development' && this.state.errorInfo && (
                    <pre className="mt-2 max-h-32 overflow-auto text-xs text-muted-foreground">
                      {this.state.errorInfo.componentStack}
                    </pre>
                  )}
                </div>
              )}

              {/* 网络状态提示 */}
              {!this.state.isOnline && (
                <div className="rounded-lg bg-orange-500/10 p-3 text-sm text-orange-600">
                  <div className="flex items-center gap-2">
                    <WifiOff className="h-4 w-4" />
                    <span>您当前处于离线状态</span>
                  </div>
                </div>
              )}

              <div className="flex flex-wrap gap-2 justify-center">
                <Button onClick={this.handleReset} variant="outline">
                  <RefreshCw className="mr-2 h-4 w-4" />
                  重试
                </Button>
                <Button onClick={this.handleReload} variant="outline">
                  <RefreshCw className="mr-2 h-4 w-4" />
                  刷新页面
                </Button>
                <Button onClick={this.handleGoHome}>
                  <Home className="mr-2 h-4 w-4" />
                  返回首页
                </Button>
              </div>

              {/* 错误代码（生产环境显示） */}
              {process.env.NODE_ENV === 'production' && this.state.error && (
                <div className="text-center text-xs text-muted-foreground">
                  错误代码: {btoa(this.state.error.message).slice(0, 20)}...
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      )
    }

    return this.props.children
  }
}

// Hook for functional components to catch async errors
export function useErrorHandler() {
  return (error: Error, context?: string) => {
    console.error('Handled error:', error)
    errorReporter.report(error, context)
  }
}

// 异步错误边界包装器
export function withErrorBoundary<P extends object>(
  Component: React.ComponentType<P>,
  errorBoundaryProps?: Omit<Props, 'children'>
) {
  return function WithErrorBoundary(props: P) {
    return (
      <ErrorBoundary {...errorBoundaryProps}>
        <Component {...props} />
      </ErrorBoundary>
    )
  }
}
