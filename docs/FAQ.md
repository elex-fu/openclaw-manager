# OpenClaw Manager 常见问题解答 (FAQ)

## 目录

- [安装问题](#安装问题)
- [配置问题](#配置问题)
- [使用问题](#使用问题)
- [故障排查](#故障排查)
- [性能问题](#性能问题)
- [其他问题](#其他问题)

---

## 安装问题

### Q1: OpenClaw Manager 支持哪些操作系统？

**A:** OpenClaw Manager 支持以下操作系统：
- **macOS**: 11.0 (Big Sur) 及以上版本，支持 Intel 和 Apple Silicon (M1/M2/M3)
- **Windows**: Windows 10 及以上版本，64 位系统
- **Linux**: Ubuntu 20.04+、Debian 11+、Fedora 35+、CentOS 8+，支持 x64 和 ARM64 架构

### Q2: 安装时提示"无法验证开发者"怎么办？

**A:**

**macOS:**
1. 打开 **系统设置 > 隐私与安全性**
2. 在"安全性"部分找到阻止 OpenClaw Manager 的提示
3. 点击 **"仍要打开"**
4. 或者按住 Control 键点击应用图标，选择"打开"

**Windows:**
1. 点击提示中的 **"更多信息"**
2. 点击 **"仍要运行"**

**注意**: 这些提示出现是因为应用没有进行代码签名。从官方渠道下载的应用是安全的。

### Q3: 如何完全卸载 OpenClaw Manager？

**A:**

**macOS:**
```bash
# 删除应用
rm -rf /Applications/OpenClaw\ Manager.app

# 删除配置和数据
rm -rf ~/Library/Application\ Support/OpenClaw\ Manager
rm -rf ~/Library/Logs/OpenClaw\ Manager
rm -rf ~/Library/Preferences/com.openclaw.manager.plist
```

**Windows:**
1. 使用"添加或删除程序"卸载
2. 手动删除 `%APPDATA%\OpenClaw Manager` 文件夹

**Linux:**
```bash
# 如果使用 AppImage，直接删除文件即可
# 如果使用 deb/rpm，使用包管理器卸载
sudo apt remove openclaw-manager  # Debian/Ubuntu
sudo rpm -e openclaw-manager       # RHEL/CentOS/Fedora

# 删除配置
rm -rf ~/.config/OpenClaw\ Manager
```

### Q4: 安装时卡在"正在下载 OpenClaw"怎么办？

**A:**
1. **检查网络连接** - 确保能够访问 GitHub
2. **配置代理** - 如果在中国大陆，可能需要配置代理
3. **使用离线安装** - 从 [Releases](https://github.com/openclaw/openclaw/releases) 手动下载 OpenClaw，然后使用"从文件安装"功能
4. **使用一键安装** - 应用支持 Molili 风格的一键安装，包含嵌入式 Runtime + OpenClaw + 国产模型预设
5. **查看日志** - 检查具体的错误信息

### Q5: 为什么安装后找不到 OpenClaw？

**A:**
- OpenClaw Manager 和 OpenClaw 是两个不同的组件
- OpenClaw Manager 是管理工具，OpenClaw 是 AI 网关服务
- 首次启动 Manager 后，需要通过"一键安装"或"在线安装"功能安装 OpenClaw
- 安装完成后，OpenClaw 通常位于 `~/.openclaw/bin/openclaw` (macOS/Linux) 或 `%USERPROFILE%\.openclaw\bin\openclaw.exe` (Windows)

### Q6: 一键安装和在线安装有什么区别？

**A:**
- **一键安装（推荐）**: 包含嵌入式 Runtime + OpenClaw + 国产模型预设，无需网络连接即可完成安装
- **在线安装**: 从网络下载最新版本的 OpenClaw，需要稳定的网络连接
- **离线安装**: 使用本地已下载的安装包进行安装

---

## 配置问题

### Q7: 如何修改 OpenClaw 的监听端口？

**A:**
1. 打开 OpenClaw Manager
2. 进入 **配置** 页面
3. 找到 **server.port** 配置项
4. 修改为想要的端口号（如 8081）
5. 点击 **保存**
6. 重启服务使配置生效

**注意**: 确保新端口未被其他程序占用。

### Q8: API 密钥存储在哪里？安全吗？

**A:**
- API 密钥使用操作系统提供的安全存储机制（keyring 库）
- **macOS**: 存储在 Keychain（钥匙串）中
- **Windows**: 存储在 Windows Credential Manager 中
- **Linux**: 存储在 Secret Service API 或 keyring 中
- 密钥不会以明文形式存储在配置文件或数据库中
- 即使他人获取了您的配置文件，也无法获取 API 密钥

### Q9: 配置文件在哪里？可以手动编辑吗？

**A:**

**配置文件位置：**
- **macOS**: `~/.openclaw/config.yaml`
- **Windows**: `%USERPROFILE%\.openclaw\config.yaml`
- **Linux**: `~/.config/openclaw/config.yaml`

**可以手动编辑**，但建议：
1. 先停止 OpenClaw 服务
2. 备份原配置文件
3. 使用 YAML 语法检查工具验证格式
4. 编辑后通过 Manager 验证配置
5. 重新启动服务

### Q10: 如何备份和恢复配置？

**A:**

**备份：**
1. 打开 OpenClaw Manager
2. 进入 **设置 > 备份**
3. 点击 **"创建备份"**
4. 选择备份位置

**恢复：**
1. 进入 **设置 > 备份**
2. 点击 **"恢复备份"**
3. 选择备份文件
4. 确认恢复

**手动备份：**
直接复制配置文件目录：
```bash
cp -r ~/.openclaw ~/.openclaw.backup.$(date +%Y%m%d)
```

### Q11: 配置验证失败怎么办？

**A:**
1. **查看具体错误** - 验证失败时会显示具体的错误信息
2. **检查必填项** - 确保所有必填配置项已填写
3. **验证 YAML 语法** - 使用在线 YAML 验证工具检查语法
4. **检查路径** - 确保配置中的文件路径存在且可访问
5. **检查端口号** - 确保端口号在有效范围内（1-65535）
6. **恢复默认** - 如果无法修复，可以重置为默认配置

---

## 使用问题

### Q12: 如何启动和停止 OpenClaw 服务？

**A:**

**通过 Manager：**
1. 打开 **控制台** 页面
2. 点击 **"启动服务"** 或 **"停止服务"** 按钮
3. 查看状态指示灯确认服务状态

**通过命令行：**
```bash
# 启动
openclaw start

# 停止
openclaw stop

# 查看状态
openclaw status
```

### Q13: 如何查看 OpenClaw 的运行日志？

**A:**

**通过 Manager（推荐）：**
1. 打开 **日志查看器** 页面
2. 日志会实时显示在界面中
3. 可以使用筛选功能查看特定级别的日志（ERROR、WARN、INFO、DEBUG）
4. 支持按日志源筛选和搜索
5. 支持导出日志为文本、JSON 或 CSV 格式

**日志文件位置：**
- **macOS**: `~/.openclaw/logs/openclaw.log`
- **Windows**: `%USERPROFILE%\.openclaw\logs\openclaw.log`
- **Linux**: `~/.local/share/openclaw/logs/openclaw.log`

**命令行查看：**
```bash
# 实时查看日志
tail -f ~/.openclaw/logs/openclaw.log
```

### Q14: 如何添加新的 AI 模型？

**A:**
1. 打开 **模型** 页面
2. 点击 **"添加模型"** 按钮
3. 选择模型提供商（如 OpenAI、Anthropic、Google、Ollama 等）
4. 填写 API 密钥（将被安全存储到系统钥匙串）
5. 选择具体的模型（如 GPT-4、Claude-3、Gemini）
6. 配置模型参数（温度、最大令牌数、top_p 等）
7. 点击 **"测试连接"** 验证配置
8. 点击 **"保存"**

### Q15: 模型连接测试失败怎么办？

**A:**
1. **检查 API 密钥** - 确保密钥正确且未过期
2. **检查网络** - 确保能够访问模型提供商的 API
3. **检查代理** - 如果使用代理，确保代理配置正确
4. **检查余额** - 确认 API 账户有足够余额
5. **查看错误信息** - 根据具体的错误代码排查：
   - 401: API 密钥无效
   - 429: 请求过于频繁或余额不足
   - 500/503: 提供商服务暂时不可用
   - ECONNREFUSED: 连接被拒绝，检查网络或防火墙
   - ETIMEDOUT: 连接超时，检查网络质量

### Q16: 如何更新 OpenClaw 到最新版本？

**A:**

**自动更新：**
1. 打开 **设置 > 更新**
2. 点击 **"检查更新"**
3. 如果有新版本，点击 **"更新"**
4. 更新过程中会显示进度，支持备份和回滚

**手动更新：**
1. 从 [Releases](https://github.com/openclaw/openclaw/releases) 下载最新版本
2. 停止 OpenClaw 服务
3. 替换旧版本二进制文件
4. 重新启动服务

**离线升级：**
- 使用本地下载的安装包进行升级
- 支持从备份恢复

### Q17: 如何使用技能商店？

**A:**
1. 打开 **技能商店** 页面
2. 浏览或搜索可用的技能
3. 点击技能查看详情（描述、版本、下载量、评分等）
4. 点击 **"安装"** 安装技能
5. 安装后可在已安装技能列表中管理（启用/禁用/配置）
6. 支持检查技能更新并一键更新

### Q18: 如何切换 Agent？

**A:**
1. 打开 **Agent** 页面
2. 查看已配置的 Agent 列表
3. 点击 **"设为当前"** 切换当前使用的 Agent
4. 或点击 **"添加 Agent"** 创建新的 Agent 配置
5. 每个 Agent 可以配置不同的模型、提示词和参数

---

## 故障排查

### Q19: 服务启动后立即退出怎么办？

**A:**
1. **查看日志** - 使用日志查看器检查错误信息
2. **检查端口占用** - 确保配置的端口未被占用
  ```bash
  # macOS/Linux
  lsof -i :8080

  # Windows
  netstat -ano | findstr :8080
  ```
3. **检查配置文件** - 验证 YAML 语法
4. **检查权限** - 确保有权限访问数据目录
5. **运行诊断** - 使用内置诊断工具检查系统环境
6. **重置配置** - 备份后删除配置文件，让系统生成默认配置

### Q20: 提示"端口已被占用"怎么办？

**A:**
1. **查找占用进程：**
   ```bash
   # macOS/Linux
   lsof -i :8080

   # Windows
   netstat -ano | findstr :8080
   tasklist | findstr <PID>
   ```
2. **终止占用进程** 或 **更换端口**
3. 在配置中修改 `server.port` 为其他端口
4. 重启服务

### Q21: 应用界面显示异常或卡顿怎么办？

**A:**
1. **重启应用** - 完全退出后重新启动
2. **清除缓存** - 删除应用缓存数据
   - macOS: `~/Library/Caches/OpenClaw Manager`
   - Windows: `%LOCALAPPDATA%\OpenClaw Manager\Cache`
3. **更新显卡驱动** - 确保显卡驱动是最新版本
4. **禁用硬件加速** - 在设置中禁用 GPU 加速
5. **查看资源占用** - 检查 CPU 和内存使用情况

### Q22: 如何重置 OpenClaw Manager 到初始状态？

**A:**

**重置应用数据（保留 OpenClaw）：**
1. 完全退出 OpenClaw Manager
2. 删除应用数据目录：
   - macOS: `~/Library/Application Support/OpenClaw Manager`
   - Windows: `%APPDATA%\OpenClaw Manager`
   - Linux: `~/.config/OpenClaw Manager`
3. 重新启动应用

**完全重置（包括 OpenClaw）：**
```bash
# 停止服务
openclaw stop

# 删除所有数据
rm -rf ~/.openclaw

# 删除 Manager 数据（参考上文）
```

### Q23: 遇到"权限 denied"错误怎么办？

**A:**

**macOS/Linux:**
```bash
# 修复数据目录权限
sudo chown -R $(whoami) ~/.openclaw

# 修复配置目录权限
sudo chown -R $(whoami) ~/.config/openclaw
```

**Windows:**
1. 右键点击应用，选择"以管理员身份运行"
2. 或修改文件夹权限：
   - 右键点击 `%USERPROFILE%\.openclaw` 文件夹
   - 属性 > 安全 > 编辑权限
   - 确保当前用户有完全控制权限

### Q24: API 密钥存储失败怎么办？

**A:**
1. **检查系统钥匙串访问权限**
   - macOS: 确保应用有访问 Keychain 的权限
   - Windows: 确保 Credential Manager 服务正在运行
   - Linux: 确保 Secret Service 或 keyring 已安装
2. **重新输入 API 密钥**
3. **检查系统存储空间**
4. **查看错误详情** - 根据具体错误代码排查

### Q25: 日志查看器无法连接怎么办？

**A:**
1. **检查日志文件是否存在** - 确认日志文件路径正确
2. **检查文件权限** - 确保有读取日志文件的权限
3. **重新初始化日志源** - 在日志查看器中点击"刷新"
4. **添加自定义日志源** - 如果日志文件在非标准位置，可手动添加
5. **检查磁盘空间** - 确保日志文件未被清理

### Q26: 技能安装失败怎么办？

**A:**
1. **检查网络连接** - 技能市场需要网络访问
2. **检查磁盘空间** - 确保有足够空间安装技能
3. **查看错误信息** - 根据具体错误代码排查
4. **重新尝试安装** - 部分错误可通过重试解决
5. **检查技能兼容性** - 确保技能与当前 OpenClaw 版本兼容

---

## 性能问题

### Q27: OpenClaw 占用内存过高怎么办？

**A:**
1. **限制并发数** - 在配置中降低 `max_concurrent_requests`
2. **调整上下文长度** - 减少 `max_context_length`
3. **启用内存限制** - 设置 `memory_limit` 配置项
4. **重启服务** - 定期重启释放内存
5. **监控内存使用** - 使用 Manager 的资源监控功能

### Q28: 如何优化 OpenClaw 的响应速度？

**A:**
1. **选择就近的模型** - 使用地理位置更近的 API 端点
2. **启用缓存** - 配置响应缓存
3. **优化模型参数** - 适当降低 `max_tokens` 和 `temperature`
4. **使用流式响应** - 启用 `stream` 模式
5. **升级硬件** - 增加内存和使用 SSD

### Q29: 为什么日志文件越来越大？

**A:**
- OpenClaw 默认会保留所有日志
- 可以配置日志轮转：
  ```yaml
  logging:
    max_size: 100MB
    max_files: 5
    level: info  # 提高日志级别减少输出
  ```
- 定期清理日志文件
- 在 Manager 中启用"自动清理日志"功能

---

## 其他问题

### Q30: OpenClaw Manager 和 OpenClaw 是什么关系？

**A:**
- **OpenClaw Manager** 是图形化管理工具，用于简化 OpenClaw 的安装、配置和管理
- **OpenClaw** 是 AI 网关服务，提供 API 接口供应用程序调用
- Manager 需要 OpenClaw 才能提供完整功能
- 可以单独使用 OpenClaw（通过命令行），但 Manager 提供了更便捷的操作方式

### Q31: 如何获取技术支持？

**A:**
- **GitHub Issues**: [提交问题](https://github.com/openclaw/openclaw-manager/issues)
- **GitHub Discussions**: [参与讨论](https://github.com/openclaw/openclaw-manager/discussions)
- **邮件**: support@openclaw.io
- **文档**: 查看 [用户指南](./USER_GUIDE.md) 和 [故障排除](./TROUBLESHOOTING.md)

### Q32: 如何贡献代码或报告 Bug？

**A:**
- 查看 [CONTRIBUTING.md](../CONTRIBUTING.md) 了解贡献指南
- 提交 Bug 报告时使用 [Issue 模板](https://github.com/openclaw/openclaw-manager/issues/new/choose)
- 提交 Pull Request 前请确保：
  - 代码通过所有测试
  - 遵循代码规范
  - 更新相关文档

### Q33: OpenClaw Manager 会收集我的数据吗？

**A:**
- **否**，OpenClaw Manager 是本地应用，不会收集用户数据
- API 密钥存储在本地系统钥匙串中
- 配置文件存储在本地
- 仅在您主动检查时才会连接网络（如下载更新、检查模型连接）
- 代码开源，可以审计数据处理方式

### Q34: 支持哪些 AI 模型提供商？

**A:**
目前支持的提供商：
- OpenAI (GPT-4, GPT-3.5, DALL-E)
- Anthropic (Claude 3, Claude 2)
- Azure OpenAI
- Google (Gemini)
- Cohere
- Mistral AI
- 本地模型 (Ollama, LM Studio, LocalAI)
- 自定义 OpenAI 兼容 API

### Q35: 可以同时配置多个模型吗？

**A:**
- **可以**，支持配置多个模型
- 可以设置模型优先级
- 支持模型故障自动切换
- 可以为不同的 Agent 指定不同的模型

### Q36: 如何导出我的所有配置？

**A:**
1. 打开 **设置 > 备份**
2. 选择 **"导出所有配置"**
3. 选择保存位置
4. 导出的文件包含：
   - OpenClaw 配置
   - 模型配置
   - Agent 配置
   - 插件配置
   - 应用设置

### Q37: 诊断工具能检测哪些问题？

**A:**
内置诊断工具可以检测：
- 系统环境兼容性
- 端口占用情况
- 配置文件有效性
- 磁盘空间
- 网络连接状态
- 服务运行状态
- API 密钥可访问性

检测后支持一键自动修复部分问题。

### Q38: 更新失败如何回滚？

**A:**
1. 打开 **设置 > 更新**
2. 点击 **"查看备份"**
3. 选择更新前的备份版本
4. 点击 **"从备份恢复"**
5. 系统会自动恢复到之前的版本和配置

---

## 快速参考

### 常用命令

```bash
# 查看版本
openclaw-manager --version
openclaw --version

# 查看帮助
openclaw-manager --help
openclaw --help

# 启动服务
openclaw start

# 停止服务
openclaw stop

# 查看状态
openclaw status

# 查看日志
tail -f ~/.openclaw/logs/openclaw.log
```

### 重要路径

| 项目 | macOS | Windows | Linux |
|------|-------|---------|-------|
| 配置 | `~/.openclaw/config.yaml` | `%USERPROFILE%\.openclaw\config.yaml` | `~/.config/openclaw/config.yaml` |
| 日志 | `~/.openclaw/logs/` | `%USERPROFILE%\.openclaw\logs\` | `~/.local/share/openclaw/logs/` |
| 数据 | `~/.openclaw/data/` | `%USERPROFILE%\.openclaw\data\` | `~/.local/share/openclaw/data/` |
| 应用数据 | `~/Library/Application Support/OpenClaw Manager` | `%APPDATA%\OpenClaw Manager` | `~/.config/OpenClaw Manager` |
| 技能 | `~/.openclaw/skills/` | `%USERPROFILE%\.openclaw\skills\` | `~/.local/share/openclaw/skills/` |

---

*FAQ 版本: 1.1.0*
*最后更新: 2026-03-03*

如果以上问题未能解决您的疑问，请通过 [GitHub Issues](https://github.com/openclaw/openclaw-manager/issues) 联系我们。
