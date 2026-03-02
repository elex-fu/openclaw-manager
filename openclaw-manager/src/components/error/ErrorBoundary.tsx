import { Component, type ErrorInfo, type ReactNode } from 'react'
import { AlertCircle, RefreshCw, Home } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'

interface Props {
  children: ReactNode
  fallback?: ReactNode
  onReset?: () => void
}

interface State {
  hasError: boolean
  error: Error | null
  errorInfo: ErrorInfo | null
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
    error: null,
    errorInfo: null,
  }

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error, errorInfo: null }
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Uncaught error:', error, errorInfo)
    this.setState({ error, errorInfo })
  }

  private handleReset = () => {
    this.setState({ hasError: false, error: null, errorInfo: null })
    this.props.onReset?.()
  }

  private handleReload = () => {
    window.location.reload()
  }

  private handleGoHome = () => {
    window.location.hash = '/'
    this.handleReset()
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
              <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-destructive/10">
                <AlertCircle className="h-8 w-8 text-destructive" />
              </div>
              <CardTitle className="text-xl">出错了</CardTitle>
              <CardDescription>
                应用程序遇到了意外错误
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
  return (error: Error) => {
    console.error('Handled error:', error)
    // Could integrate with error reporting service here
  }
}
