# OpenClaw Manager 测试体系设计

## 一、当前系统状态分析

### 1.1 前端架构
```
技术栈: React 18 + TypeScript + Vite + TanStack Query + Zustand + shadcn/ui
测试框架: Vitest (推荐) / Jest
E2E框架: Playwright
```

### 1.2 后端架构
```
技术栈: Rust + Tauri v2 + SQLite
测试框架: cargo test
```

### 1.3 当前存在的问题

#### 前端问题
| 问题 | 数量 | 严重程度 | 影响 |
|------|------|----------|------|
| TypeScript类型警告 | 25 | 低 | 不影响运行，影响代码质量 |
| 未使用的import/变量 | 15 | 低 | 代码冗余 |
| API返回类型访问错误 | 5 | 中 | 可能运行时错误 |
| 缺少模型属性 | 3 | 中 | 类型不完整 |

#### 后端问题
| 问题 | 状态 | 影响 |
|------|------|------|
| Rust工具链安装中 | 阻塞 | 无法编译验证 |
| 未验证的Tauri命令 | 未知 | 可能运行时报错 |

---

## 二、测试用例设计

### 2.1 单元测试 (Unit Tests)

#### 前端单元测试

##### A. Store 测试

**appStore.test.ts**
```typescript
describe('appStore', () => {
  describe('notifications', () => {
    it('should add notification with auto-generated id', () => {
      // 测试通知添加
    });
    
    it('should auto-remove notification after 5s', () => {
      // 测试自动移除
    });
    
    it('should limit max notifications to 5', () => {
      // 测试最大数量限制
    });
  });
  
  describe('theme', () => {
    it('should persist theme preference', () => {
      // 测试主题持久化
    });
  });
});
```

**installStore.test.ts**
```typescript
describe('installStore', () => {
  describe('install progress', () => {
    it('should track installation progress', () => {
      // 测试进度跟踪
    });
    
    it('should reset state on complete', () => {
      // 测试重置功能
    });
  });
  
  describe('wizard steps', () => {
    it('should navigate between steps', () => {
      // 测试步骤导航
    });
    
    it('should not go below step 0', () => {
      // 测试边界
    });
  });
});
```

**configStore.test.ts**
```typescript
describe('configStore', () => {
  describe('models', () => {
    it('should add model', () => {});
    it('should update model', () => {});
    it('should remove model', () => {});
    it('should set default model', () => {});
    it('should reorder models', () => {});
  });
  
  describe('agents', () => {
    it('should add agent', () => {});
    it('should update agent', () => {});
    it('should set current agent', () => {});
  });
  
  describe('apiKeyCache', () => {
    it('should not persist apiKeyCache', () => {
      // 测试敏感数据不持久化
    });
  });
});
```

##### B. API 层测试

**tauri-api.test.ts**
```typescript
describe('tauri-api', () => {
  describe('openclawApi', () => {
    it('should check installation status', async () => {});
    it('should install openclaw', async () => {});
    it('should handle install error', async () => {});
    it('should listen to install progress', async () => {});
  });
  
  describe('secureStorageApi', () => {
    it('should save api key', async () => {});
    it('should get api key', async () => {});
    it('should delete api key', async () => {});
    it('should check if api key exists', async () => {});
  });
  
  describe('serviceApi', () => {
    it('should start service', async () => {});
    it('should stop service', async () => {});
    it('should get service status', async () => {});
    it('should perform health check', async () => {});
  });
  
  describe('modelApi', () => {
    it('should get all models', async () => {});
    it('should save model', async () => {});
    it('should test model connection', async () => {});
  });
  
  describe('agentApi', () => {
    it('should get all agents', async () => {});
    it('should save agent', async () => {});
    it('should set current agent', async () => {});
  });
  
  describe('diagnosticsApi', () => {
    it('should run diagnostics', async () => {});
    it('should auto fix issues', async () => {});
    it('should fix single issue', async () => {});
  });
});
```

##### C. 工具函数测试

**utils.test.ts**
```typescript
describe('utils', () => {
  describe('cn (className merge)', () => {
    it('should merge classNames', () => {});
    it('should handle conditional classes', () => {});
  });
});
```

#### 后端单元测试

##### A. 错误处理测试

**app_error.test.rs**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_error_to_user_message() {
        // 测试错误转换为用户友好消息
        let error = AppError::Install(InstallError::DownloadFailed("network".into()));
        let msg = error.to_user_message();
        assert_eq!(msg.severity, ErrorSeverity::Error);
        assert!(msg.retryable);
    }
    
    #[test]
    fn test_error_serialization() {
        // 测试错误序列化
    }
}
```

##### B. 重试机制测试

**retry.test.rs**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_retry_with_backoff_success() {
        // 测试成功不重试
    }
    
    #[tokio::test]
    async fn test_retry_with_backoff_eventual_success() {
        // 测试最终成功
    }
    
    #[tokio::test]
    async fn test_retry_with_backoff_max_attempts() {
        // 测试最大重试次数
    }
    
    #[tokio::test]
    async fn test_exponential_backoff() {
        // 测试指数退避
    }
}
```

##### C. 安全存储测试

**secure_storage.test.rs**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_save_and_get_api_key() {
        // 测试保存和获取
    }
    
    #[test]
    fn test_delete_api_key() {
        // 测试删除
    }
    
    #[test]
    fn test_has_api_key() {
        // 测试检查存在性
    }
    
    #[test]
    fn test_get_nonexistent_key() {
        // 测试获取不存在的key
    }
}
```

##### D. 离线安装器测试

**offline_installer.test.rs**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_platform() {
        // 测试平台检测
    }
    
    #[test]
    fn test_package_info_creation() {
        // 测试包信息创建
    }
    
    #[tokio::test]
    async fn test_extract_package_tar_gz() {
        // 测试tar.gz解压
    }
    
    #[tokio::test]
    async fn test_extract_package_zip() {
        // 测试zip解压
    }
}
```

##### E. 配置管理器测试

**config_manager.test.rs**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_validation() {
        // 测试配置验证
    }
    
    #[test]
    fn test_optimistic_locking() {
        // 测试乐观锁
    }
    
    #[test]
    fn test_import_export() {
        // 测试导入导出
    }
}
```

##### F. 进程管理器测试

**process_manager.test.rs**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_start_service() {
        // 测试启动服务
    }
    
    #[tokio::test]
    async fn test_stop_service_graceful() {
        // 测试优雅停止
    }
    
    #[tokio::test]
    async fn test_health_check() {
        // 测试健康检查
    }
    
    #[tokio::test]
    async fn test_service_events() {
        // 测试服务事件广播
    }
}
```

---

### 2.2 集成测试 (Integration Tests)

#### 前端集成测试

**pages-integration.test.tsx**
```typescript
describe('Page Integration', () => {
  describe('Dashboard', () => {
    it('should fetch and display installation status', async () => {});
    it('should display service status', async () => {});
    it('should navigate to install wizard when not installed', async () => {});
  });
  
  describe('InstallWizard', () => {
    it('should complete full installation flow', async () => {});
    it('should handle installation errors', async () => {});
    it('should allow offline installation selection', async () => {});
  });
  
  describe('ModelConfig', () => {
    it('should save model with api key', async () => {});
    it('should test model connection', async () => {});
    it('should set default model', async () => {});
  });
  
  describe('AgentManager', () => {
    it('should create and switch agent', async () => {});
    it('should persist agent changes', async () => {});
  });
  
  describe('Diagnostics', () => {
    it('should run full diagnostics', async () => {});
    it('should auto-fix issues', async () => {});
  });
});
```

#### 后端集成测试

**commands_integration.test.rs**
```rust
#[cfg(test)]
mod tests {
    // 测试 Tauri 命令集成
    
    #[tokio::test]
    async fn test_install_openclaw_command() {
        // 测试安装命令
    }
    
    #[tokio::test]
    async fn test_model_api_commands() {
        // 测试模型相关命令
    }
    
    #[tokio::test]
    async fn test_secure_storage_commands() {
        // 测试安全存储命令
    }
    
    #[tokio::test]
    async fn test_service_commands() {
        // 测试服务控制命令
    }
}
```

---

### 2.3 E2E 测试

**e2e/installation.spec.ts**
```typescript
import { test, expect } from '@playwright/test';

test.describe('Installation Flow', () => {
  test('should install OpenClaw from scratch', async ({ page }) => {
    // 1. 打开应用
    // 2. 检查未安装状态
    // 3. 进入安装向导
    // 4. 完成环境检测
    // 5. 选择安装方式
    // 6. 等待安装完成
    // 7. 验证已安装状态
  });
  
  test('should handle offline installation', async ({ page }) => {
    // 测试离线安装流程
  });
  
  test('should handle installation errors', async ({ page }) => {
    // 测试错误处理
  });
});
```

**e2e/model-config.spec.ts**
```typescript
test.describe('Model Configuration', () => {
  test('should add and configure OpenAI model', async ({ page }) => {
    // 1. 导航到模型配置页面
    // 2. 点击添加模型
    // 3. 填写模型信息
    // 4. 输入 API Key
    // 5. 测试连接
    // 6. 保存模型
    // 7. 验证模型列表
  });
  
  test('should set default model', async ({ page }) => {
    // 测试设置默认模型
  });
  
  test('should secure store api key', async ({ page }) => {
    // 验证 API Key 安全存储
  });
});
```

**e2e/agent-management.spec.ts**
```typescript
test.describe('Agent Management', () => {
  test('should create and switch agent', async ({ page }) => {
    // 测试 Agent 创建和切换
  });
});
```

**e2e/service-control.spec.ts**
```typescript
test.describe('Service Control', () => {
  test('should start and stop gateway service', async ({ page }) => {
    // 测试服务控制
  });
  
  test('should display service status', async ({ page }) => {
    // 测试状态显示
  });
});
```

**e2e/diagnostics.spec.ts**
```typescript
test.describe('Diagnostics', () => {
  test('should run diagnostics and display results', async ({ page }) => {
    // 测试诊断功能
  });
  
  test('should auto-fix issues', async ({ page }) => {
    // 测试自动修复
  });
});
```

---

## 三、测试实施计划

### 阶段1: 基础测试设施 (1天)
- [ ] 配置 Vitest (前端)
- [ ] 配置 Playwright (E2E)
- [ ] 设置测试目录结构
- [ ] 添加测试脚本到 package.json

### 阶段2: 核心功能单元测试 (2天)
- [ ] Store 单元测试
- [ ] API 层测试
- [ ] 工具函数测试
- [ ] Rust 核心业务测试

### 阶段3: 集成测试 (2天)
- [ ] 前端页面集成测试
- [ ] Rust 命令集成测试
- [ ] 前后端联调测试

### 阶段4: E2E 测试 (2天)
- [ ] 安装流程 E2E
- [ ] 配置管理 E2E
- [ ] 服务控制 E2E
- [ ] 诊断修复 E2E

### 阶段5: CI/CD 集成 (1天)
- [ ] GitHub Actions 配置
- [ ] 自动化测试流程
- [ ] 测试覆盖率报告

---

## 四、当前问题清单

### 阻塞性问题
| 问题 | 状态 | 解决方案 |
|------|------|----------|
| Rust 工具链安装中 | 🔴 阻塞 | 等待安装完成或手动安装 |

### 高优先级问题
| 问题 | 影响 | 解决方案 |
|------|------|----------|
| TypeScript API 返回类型访问错误 | 运行时可能出错 | 统一 API 响应类型 |
| 模型定义不完整 | 类型检查失败 | 补全类型定义 |
| 组件 Props 类型不匹配 | 类型检查失败 | 修正 Props 定义 |

### 中优先级问题
| 问题 | 影响 | 解决方案 |
|------|------|----------|
| 未使用的 import/变量 | 代码冗余 | 清理代码 |
| Store 中未使用的 get | 代码冗余 | 移除或添加使用 |
| 缺少依赖 @radix-ui/react-separator | 组件无法使用 | 已安装 |

### 低优先级问题
| 问题 | 影响 | 解决方案 |
|------|------|----------|
| 代码格式不一致 | 可读性 | 使用 Prettier |
| 缺少注释 | 可维护性 | 添加文档注释 |

---

## 五、测试覆盖率目标

| 模块 | 单元测试 | 集成测试 | E2E | 目标覆盖率 |
|------|---------|---------|-----|-----------|
| Stores | ✅ | - | - | 90% |
| API Layer | ✅ | ✅ | - | 85% |
| Components | ✅ | ✅ | - | 80% |
| Pages | - | ✅ | ✅ | 75% |
| Rust Services | ✅ | ✅ | - | 85% |
| Rust Commands | - | ✅ | ✅ | 80% |

---

## 六、推荐测试工具

### 前端
- **单元测试**: Vitest (速度快，Vite 原生支持)
- **组件测试**: React Testing Library
- **E2E测试**: Playwright (跨浏览器，Tauri 官方推荐)
- **覆盖率**: v8 (Vitest 内置)

### 后端
- **单元测试**: cargo test (Rust 内置)
- **覆盖率**: tarpaulin / cargo-llvm-cov
- **Mock**: mockall

---

*文档版本: 1.0*  
*创建时间: 2026-02-26*  
*更新: 待测试实施完成后更新*
