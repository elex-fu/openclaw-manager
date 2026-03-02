import { useState, useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { Loader2 } from 'lucide-react'
import type { InstalledSkill } from '@/types'

interface SkillConfigFormProps {
  skill: InstalledSkill
  onSubmit: (config: Record<string, unknown>) => void
  isLoading?: boolean
}

export function SkillConfigForm({ skill, onSubmit, isLoading = false }: SkillConfigFormProps) {
  const [config, setConfig] = useState<Record<string, unknown>>(skill.config || {})
  const [errors, setErrors] = useState<Record<string, string>>({})

  const schema = skill.config_schema as Record<string, unknown> | undefined
  const properties = (schema?.properties || {}) as Record<string, Record<string, unknown>>
  const required = (schema?.required || []) as string[]

  // 验证配置
  const validate = (): boolean => {
    const newErrors: Record<string, string> = {}

    for (const key of required) {
      const value = config[key]
      if (value === undefined || value === null || value === '') {
        newErrors[key] = '此项为必填项'
      }
    }

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  // 处理提交
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (validate()) {
      onSubmit(config)
    }
  }

  // 更新配置值
  const updateValue = (key: string, value: unknown) => {
    setConfig((prev) => ({ ...prev, [key]: value }))
    // 清除该字段的错误
    if (errors[key]) {
      setErrors((prev) => {
        const newErrors = { ...prev }
        delete newErrors[key]
        return newErrors
      })
    }
  }

  // 渲染表单字段
  const renderField = (key: string, prop: Record<string, unknown>) => {
    const type = prop.type as string
    const title = (prop.title as string) || key
    const description = prop.description as string | undefined
    const isRequired = required.includes(key)
    const value = config[key]
    const error = errors[key]

    switch (type) {
      case 'string':
        if (prop.enum) {
          // 枚举类型使用 Select
          return (
            <div key={key} className="space-y-2">
              <Label htmlFor={key}>
                {title}
                {isRequired && <span className="text-destructive ml-1">*</span>}
              </Label>
              <Select
                value={(value as string) || ''}
                onValueChange={(v) => updateValue(key, v)}
              >
                <SelectTrigger id={key} className={error ? 'border-destructive' : ''}>
                  <SelectValue placeholder={`选择 ${title}`} />
                </SelectTrigger>
                <SelectContent>
                  {(prop.enum as string[]).map((option) => (
                    <SelectItem key={option} value={option}>
                      {option}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {description && (
                <p className="text-xs text-muted-foreground">{description}</p>
              )}
              {error && <p className="text-xs text-destructive">{error}</p>}
            </div>
          )
        }
        // 普通字符串使用 Input
        return (
          <div key={key} className="space-y-2">
            <Label htmlFor={key}>
              {title}
              {isRequired && <span className="text-destructive ml-1">*</span>}
            </Label>
            <Input
              id={key}
              value={(value as string) || ''}
              onChange={(e) => updateValue(key, e.target.value)}
              placeholder={description}
              className={error ? 'border-destructive' : ''}
            />
            {error && <p className="text-xs text-destructive">{error}</p>}
          </div>
        )

      case 'number':
      case 'integer':
        return (
          <div key={key} className="space-y-2">
            <Label htmlFor={key}>
              {title}
              {isRequired && <span className="text-destructive ml-1">*</span>}
            </Label>
            <Input
              id={key}
              type="number"
              value={(value as number) ?? ''}
              onChange={(e) => updateValue(key, parseFloat(e.target.value))}
              placeholder={description}
              className={error ? 'border-destructive' : ''}
            />
            {error && <p className="text-xs text-destructive">{error}</p>}
          </div>
        )

      case 'boolean':
        return (
          <div key={key} className="flex items-center justify-between space-y-0 py-2">
            <div className="space-y-0.5">
              <Label htmlFor={key}>
                {title}
                {isRequired && <span className="text-destructive ml-1">*</span>}
              </Label>
              {description && (
                <p className="text-xs text-muted-foreground">{description}</p>
              )}
            </div>
            <Switch
              id={key}
              checked={(value as boolean) || false}
              onCheckedChange={(v) => updateValue(key, v)}
            />
          </div>
        )

      default:
        // 其他类型使用文本区域
        return (
          <div key={key} className="space-y-2">
            <Label htmlFor={key}>
              {title}
              {isRequired && <span className="text-destructive ml-1">*</span>}
            </Label>
            <Textarea
              id={key}
              value={JSON.stringify(value, null, 2)}
              onChange={(e) => {
                try {
                  const parsed = JSON.parse(e.target.value)
                  updateValue(key, parsed)
                } catch {
                  updateValue(key, e.target.value)
                }
              }}
              placeholder={description}
              className={error ? 'border-destructive' : ''}
              rows={3}
            />
            {error && <p className="text-xs text-destructive">{error}</p>}
          </div>
        )
    }
  }

  // 如果没有配置schema
  if (!schema || Object.keys(properties).length === 0) {
    return (
      <div className="space-y-4">
        <p className="text-muted-foreground">此技能没有可配置的选项。</p>
        <div className="flex justify-end">
          <Button onClick={() => onSubmit({})} disabled={isLoading}>
            {isLoading ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                保存中...
              </>
            ) : (
              '确定'
            )}
          </Button>
        </div>
      </div>
    )
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      <div className="space-y-4">
        {Object.entries(properties).map(([key, prop]) => renderField(key, prop))}
      </div>

      <div className="flex justify-end gap-2">
        <Button
          type="button"
          variant="outline"
          onClick={() => setConfig(skill.default_config || {})}
          disabled={isLoading}
        >
          重置默认
        </Button>
        <Button type="submit" disabled={isLoading}>
          {isLoading ? (
            <>
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              保存中...
            </>
          ) : (
            '保存配置'
          )}
        </Button>
      </div>
    </form>
  )
}
