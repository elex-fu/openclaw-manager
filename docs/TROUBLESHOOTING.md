# OpenClaw Manager 故障排除指南

本指南帮助您诊断和解决 OpenClaw Manager 使用过程中遇到的常见问题。

## 目录

- [快速诊断](#快速诊断)
- [安装问题](#安装问题)
- [启动问题](#启动问题)
- [服务问题](#服务问题)
- [配置问题](#配置问题)
- [模型连接问题](#模型连接问题)
- [API 密钥存储问题](#api-密钥存储问题)
- [Agent 管理问题](#agent-管理问题)
- [日志查看器问题](#日志查看器问题)
- [技能商店问题](#技能商店问题)
- [更新系统问题](#更新系统问题)
- [诊断系统问题](#诊断系统问题)
- [性能问题](#性能问题)
- [界面问题](#界面问题)
- [日志查看方法](#日志查看方法)
- [重置配置方法](#重置配置方法)
- [获取帮助](#获取帮助)

---

## 快速诊断

遇到问题时，请按照以下步骤进行初步诊断：

### 1. 检查系统要求

确保您的系统满足最低要求：
- **操作系统**: macOS 11+ / Windows 10+ / Ubuntu 20.04+
- **内存**: 至少 4GB 可用内存
- **磁盘**: 至少 1GB 可用空间
- **网络**: 能够访问互联网（用于下载和模型连接）

### 2. 查看错误信息

- 记录具体的错误提示信息
- 截图错误对话框
- 注意错误发生的时间和操作步骤

### 3. 检查日志

查看日志是诊断问题的关键步骤：
```bash
# 查看应用日志
tail -n 100 ~/.openclaw/logs/openclaw.log

# 查看 Manager 日志（macOS）
tail -n 100 ~/Library/Logs/OpenClaw\ Manager/log.log
```

### 4. 运行诊断工具

OpenClaw Manager 内置了诊断工具：
1. 打开 **诊断** 页面
2. 点击 **"运行诊断"**
3. 查看检测结果
4. 对可修复问题点击 **"一键修复"**

### 5. 尝试基本修复

- 重启应用
- 重启电脑
- 检查网络连接
- 暂时禁用防火墙/杀毒软件测试

---

## 安装问题

### 问题：安装程序无法运行

**症状**：
- 双击安装程序无反应
- 提示"无法打开，因为无法验证开发者"
- 提示"此应用无法在你的电脑上运行"

**解决方案**：

**macOS:**
```bash
# 移除隔离属性
xattr -d com.apple.quarantine /Applications/OpenClaw\ Manager.app

# 或允许任何来源的应用（不推荐长期使用）
sudo spctl --master-disable
```

**Windows:**
1. 右键点击安装程序，选择"属性"
2. 勾选"解除锁定"（如果有）
3. 点击"确定"
4. 再次运行安装程序

**Linux:**
```bash
# 确保有执行权限
chmod +x openclaw-manager.AppImage

# 安装 FUSE（AppImage 需要）
# Ubuntu/Debian
sudo apt install libfuse2

# Fedora
sudo dnf install fuse
```

### 问题：下载 OpenClaw 失败

**症状**：
- 卡在"正在下载"步骤
- 提示网络错误
- 下载速度极慢

**解决方案**：

1. **检查网络连接**
   ```bash
   # 测试 GitHub 连接
   curl -I https://github.com
   ```

2. **配置代理**（如需要）
   ```bash
   # 设置环境变量
   export HTTP_PROXY=http://proxy.example.com:8080
   export HTTPS_PROXY=http://proxy.example.com:8080
   ```

3. **使用离线安装**
   - 从 [Releases](https://github.com/openclaw/openclaw/releases) 手动下载
   - 在 Manager 中选择"从文件安装"

4. **使用一键安装**
   - 选择"一键安装"模式，包含嵌入式 Runtime，无需网络下载

5. **更换下载源**
   - 在设置中配置镜像源
   - 或使用国内镜像

### 问题：磁盘空间不足

**症状**：
- 提示"磁盘空间不足"
- 安装过程中断

**解决方案**：

```bash
# 检查磁盘空间
df -h

# 清理临时文件
# macOS
rm -rf ~/Library/Caches/OpenClaw\ Manager

# Windows
# 运行磁盘清理工具

# Linux
rm -rf ~/.cache/OpenClaw\ Manager
```

### 问题：一键安装失败

**症状**：
- 一键安装过程中断
- 提示"安装脚本失败"
- 提示"Package not found"

**解决方案**：

1. **检查系统环境**
   ```bash
   # 运行系统环境检查
   openclaw-manager --check-environment
   ```

2. **清理残留文件**
   ```bash
   rm -rf ~/.openclaw/tmp
   rm -rf ~/.openclaw/downloads
   ```

3. **重新尝试**
   - 关闭其他占用资源的程序
   - 重新启动一键安装

4. **切换到在线安装**
   - 如果一键安装持续失败，尝试使用在线安装模式

---

## 启动问题

### 问题：应用无法启动

**症状**：
- 点击图标无反应
- 启动后立即崩溃
- 显示白屏或黑屏

**诊断步骤**：

1. **检查日志**
   ```bash
   # macOS
   cat ~/Library/Logs/OpenClaw\ Manager/log.log

   # Windows
   type %APPDATA%\OpenClaw Manager\logs\log.log

   # Linux
   cat ~/.config/OpenClaw\ Manager/logs/log.log
   ```

2. **重置应用数据**
   ```bash
   # 完全退出应用后执行

   # macOS
   rm -rf ~/Library/Application\ Support/OpenClaw\ Manager

   # Windows
   rmdir /s "%APPDATA%\OpenClaw Manager"

   # Linux
   rm -rf ~/.config/OpenClaw\ Manager
   ```

3. **检查冲突软件**
   - 暂时关闭杀毒软件
   - 关闭其他可能冲突的应用
   - 检查系统代理设置

### 问题：启动缓慢

**症状**：
- 启动时间超过 30 秒
- 界面卡顿

**解决方案**：

1. **清理缓存**
   ```bash
   # macOS
   rm -rf ~/Library/Caches/OpenClaw\ Manager

   # Windows
   rmdir /s "%LOCALAPPDATA%\OpenClaw Manager\Cache"

   # Linux
   rm -rf ~/.cache/OpenClaw\ Manager
   ```

2. **禁用不必要的启动项**
   - 在设置中禁用"启动时检查更新"
   - 禁用"启动时自动启动服务"

3. **优化系统**
   - 关闭不必要的应用
   - 增加可用内存

---

## 服务问题

### 问题：服务无法启动

**症状**：
- 点击"启动服务"无反应
- 服务启动后立即停止
- 状态指示灯保持灰色

**诊断步骤**：

1. **检查端口占用**
   ```bash
   # 查看端口占用（默认 8080）
   # macOS/Linux
   lsof -i :8080

   # Windows
   netstat -ano | findstr :8080

   # 终止占用进程
   # macOS/Linux
   kill -9 <PID>

   # Windows
   taskkill /PID <PID> /F
   ```

2. **检查配置文件**
   ```bash
   # 验证 YAML 语法
   # 安装 yamllint
   pip install yamllint

   # 检查配置
   yamllint ~/.openclaw/config.yaml
   ```

3. **检查权限**
   ```bash
   # 确保有权限访问数据目录
   ls -la ~/.openclaw

   # 修复权限
   chmod -R 755 ~/.openclaw
   ```

4. **查看详细日志**
   ```bash
   # 手动运行查看错误
   ~/.openclaw/bin/openclaw start --verbose
   ```

5. **运行诊断工具**
   - 打开诊断页面
   - 运行系统诊断
   - 查看服务相关检测结果

### 问题：服务启动后立即退出

**症状**：
- 服务状态快速从"启动中"变为"已停止"
- 日志中出现错误后服务停止

**常见原因及解决方案**：

**1. 配置错误**
```bash
# 重置配置
cd ~/.openclaw
mv config.yaml config.yaml.backup
# 重启 Manager，会自动生成默认配置
```

**2. 数据目录损坏**
```bash
# 备份数据
cp -r ~/.openclaw/data ~/.openclaw/data.backup

# 重置数据
rm -rf ~/.openclaw/data/*
```

**3. 依赖缺失**
```bash
# 重新安装 OpenClaw
openclaw-manager
# 在界面中选择重新安装
```

**4. 端口冲突**
- 修改配置文件中的端口号
- 或终止占用端口的进程

### 问题：服务无法停止

**症状**：
- 点击"停止服务"无反应
- 服务进程仍在运行

**解决方案**：

```bash
# 强制停止服务
# macOS/Linux
pkill -f openclaw

# Windows
taskkill /F /IM openclaw.exe

# 或者查找并终止特定进程
# macOS/Linux
ps aux | grep openclaw
kill -9 <PID>
```

---

## 配置问题

### 问题：配置保存失败

**症状**：
- 点击保存后提示错误
- 配置更改未生效

**解决方案**：

1. **检查配置文件权限**
   ```bash
   # 确保有写入权限
   ls -la ~/.openclaw/config.yaml

   # 修复权限
   chmod 644 ~/.openclaw/config.yaml
   ```

2. **验证 YAML 语法**
   - 使用在线 YAML 验证工具
   - 检查缩进（使用空格，不是 Tab）
   - 检查特殊字符

3. **检查磁盘空间**
   ```bash
   df -h ~/.openclaw
   ```

4. **检查配置版本**
   - 如果提示"Configuration version mismatch"，需要更新配置格式
   - 备份后删除旧配置，让系统生成默认配置

### 问题：配置验证失败

**症状**：
- 提示配置验证错误
- 无法保存特定配置项

**常见错误及修复**：

**错误：Invalid port number**
```yaml
# 错误
server:
  port: 99999  # 超出范围

# 正确
server:
  port: 8080  # 1-65535 之间
```

**错误：Path does not exist**
```yaml
# 错误
storage:
  data_dir: /nonexistent/path

# 正确
storage:
  data_dir: ~/.openclaw/data  # 确保路径存在
```

**错误：Invalid YAML syntax**
```yaml
# 错误 - 混合使用 Tab 和空格
server:
	port: 8080

# 正确 - 统一使用空格（2或4个）
server:
  port: 8080
```

**错误：Configuration is locked**
```bash
# 检查是否有其他进程正在使用配置
lsof ~/.openclaw/config.yaml

# 终止占用进程或重启系统
```

---

## 模型连接问题

### 问题：模型连接测试失败

**症状**：
- 测试连接时提示错误
- 无法使用模型进行对话

**诊断步骤**：

1. **检查 API 密钥**
   - 确认密钥正确无误
   - 检查密钥是否过期
   - 确认密钥有足够的余额/配额

2. **检查网络连接**
   ```bash
   # 测试 API 端点
   # OpenAI
   curl https://api.openai.com/v1/models \
     -H "Authorization: Bearer YOUR_API_KEY"

   # Anthropic
   curl https://api.anthropic.com/v1/models \
     -H "x-api-key: YOUR_API_KEY"
   ```

3. **检查代理设置**
   ```yaml
   # 在配置中添加代理
   network:
     proxy:
       http: http://proxy.example.com:8080
       https: http://proxy.example.com:8080
   ```

4. **检查模型参数**
   - temperature: 必须在 0-2 之间
   - max_tokens: 必须在 1-8192 之间
   - top_p: 必须在 0-1 之间

### 常见错误代码

**401 - Unauthorized**
- 原因：API 密钥无效
- 解决：检查并重新输入 API 密钥

**429 - Too Many Requests**
- 原因：请求频率过高或余额不足
- 解决：降低请求频率或充值账户

**500/503 - Server Error**
- 原因：提供商服务暂时不可用
- 解决：稍后重试或切换到备用模型

**ECONNREFUSED / Connection refused**
- 原因：无法连接到 API 端点
- 解决：检查网络连接和防火墙设置

**ETIMEDOUT / Timeout**
- 原因：连接超时
- 解决：检查网络质量，增加超时时间

---

## API 密钥存储问题

### 问题：API 密钥保存失败

**症状**：
- 保存 API 密钥时提示错误
- 提示"Secure storage error"

**解决方案**：

**macOS:**
```bash
# 检查 Keychain 访问权限
# 1. 打开"钥匙串访问"应用
# 2. 检查是否有 OpenClaw Manager 的条目
# 3. 确保应用有访问权限

# 重置钥匙串权限
security delete-keychain ~/Library/Keychains/openclaw.keychain-db 2>/dev/null || true
```

**Windows:**
```powershell
# 检查 Credential Manager 服务
Get-Service VaultSvc

# 确保服务正在运行
Start-Service VaultSvc
```

**Linux:**
```bash
# 检查 Secret Service 是否安装
# Ubuntu/Debian
sudo apt install gnome-keyring libsecret-1-0

# 检查 keyring 是否正常工作
python3 -c "import keyring; print(keyring.get_keyring())"
```

### 问题：无法读取已保存的 API 密钥

**症状**：
- 提示"Access denied to secure storage"
- 密钥显示为空

**解决方案**：

1. **检查系统权限**
   - 确保当前用户有访问系统密钥存储的权限
   - 在企业环境中，可能需要联系 IT 管理员

2. **重新保存密钥**
   - 删除旧密钥
   - 重新输入并保存

3. **检查密钥存储状态**
   ```bash
   # macOS
   security find-generic-password -s "openclaw" 2>&1 | head -5
   ```

---

## Agent 管理问题

### 问题：无法切换 Agent

**症状**：
- 点击切换 Agent 无反应
- 提示"设置当前 Agent 失败"

**解决方案**：

1. **检查 Agent 配置**
   - 确保 Agent ID 正确
   - 检查 Agent 配置是否完整

2. **检查配置文件权限**
   ```bash
   ls -la ~/.openclaw/config.yaml
   chmod 644 ~/.openclaw/config.yaml
   ```

3. **重新设置当前 Agent**
   ```bash
   # 通过命令行设置
   openclaw config set agent.current <agent_id>
   ```

4. **重置 Agent 配置**
   - 备份配置
   - 删除 Agent 相关配置
   - 重新创建 Agent

### 问题：Agent 配置保存失败

**症状**：
- 保存 Agent 时提示错误
- 新创建的 Agent 不显示

**解决方案**：

1. **验证 Agent 配置**
   - 确保 Agent ID 唯一
   - 检查必填字段是否填写

2. **检查磁盘空间**
   ```bash
   df -h ~/.openclaw
   ```

3. **手动编辑配置**
   ```bash
   # 备份配置
cp ~/.openclaw/config.yaml ~/.openclaw/config.yaml.backup

   # 编辑配置
   nano ~/.openclaw/config.yaml
   ```

---

## 日志查看器问题

### 问题：日志查看器无法连接

**症状**：
- 日志查看器显示空白
- 提示"无法连接到日志源"

**解决方案**：

1. **检查日志文件是否存在**
   ```bash
   ls -la ~/.openclaw/logs/
   ```

2. **检查文件权限**
   ```bash
   chmod 644 ~/.openclaw/logs/*.log
   ```

3. **重新初始化日志源**
   - 在日志查看器中点击"刷新"
   - 或重启应用

4. **添加自定义日志源**
   - 如果日志文件在非标准位置
   - 使用"添加日志源"功能手动添加

### 问题：实时日志不更新

**症状**：
- 日志查看器卡在旧日志
- 新日志不显示

**解决方案**：

1. **检查订阅状态**
   - 确保日志订阅已建立
   - 重新订阅日志

2. **检查日志级别筛选**
   - 确保选中了正确的日志级别
   - 尝试选择"全部"级别

3. **检查日志源筛选**
   - 确保选中了正确的日志源

4. **重启日志服务**
   ```bash
   # 重启 OpenClaw 服务
   openclaw restart
   ```

### 问题：日志导出失败

**症状**：
- 导出日志时提示错误
- 导出的文件为空

**解决方案**：

1. **检查导出路径权限**
   - 确保有写入目标目录的权限
   - 尝试导出到其他目录

2. **检查磁盘空间**
   ```bash
   df -h
   ```

3. **减少导出范围**
   - 使用筛选器减少日志数量
   - 缩短时间范围

---

## 技能商店问题

### 问题：无法加载技能市场

**症状**：
- 技能商店页面空白
- 提示"无法连接到技能市场"

**解决方案**：

1. **检查网络连接**
   ```bash
   curl -I https://market.openclaw.io
   ```

2. **检查代理设置**
   - 如果使用代理，确保配置正确

3. **清除缓存**
   ```bash
   rm -rf ~/.openclaw/cache/market
   ```

4. **重试加载**
   - 点击"刷新"按钮
   - 或重启应用

### 问题：技能安装失败

**症状**：
- 安装技能时提示错误
- 安装进度卡住

**解决方案**：

1. **检查网络连接**
   - 确保能够访问技能下载地址

2. **检查磁盘空间**
   ```bash
   df -h ~/.openclaw/skills
   ```

3. **检查技能兼容性**
   - 查看技能要求的 OpenClaw 版本
   - 确保当前版本满足要求

4. **手动安装**
   - 下载技能包
   - 解压到 `~/.openclaw/skills/` 目录

### 问题：技能更新失败

**症状**：
- 检查更新时出错
- 更新技能时失败

**解决方案**：

1. **检查网络连接**
2. **重新尝试**
   - 部分更新错误可通过重试解决

3. **手动更新**
   - 卸载旧版本
   - 安装新版本

---

## 更新系统问题

### 问题：检查更新失败

**症状**：
- 点击"检查更新"无反应
- 提示"无法连接到更新服务器"

**解决方案**：

1. **检查网络连接**
   ```bash
   curl -I https://api.github.com/repos/openclaw/openclaw/releases/latest
   ```

2. **检查代理设置**
3. **手动检查更新**
   - 访问 [Releases](https://github.com/openclaw/openclaw/releases) 页面
   - 查看最新版本

### 问题：下载更新失败

**症状**：
- 更新下载过程中断
- 提示"Download failed"

**解决方案**：

1. **检查网络稳定性**
2. **更换网络环境**
3. **使用离线更新**
   - 手动下载更新包
   - 使用"从文件更新"功能

4. **清理下载缓存**
   ```bash
   rm -rf ~/.openclaw/updates/downloads
   ```

### 问题：更新安装失败

**症状**：
- 更新下载成功但安装失败
- 提示"Installation failed"

**解决方案**：

1. **检查磁盘空间**
   ```bash
   df -h
   ```

2. **检查文件权限**
   ```bash
   ls -la ~/.openclaw/
   ```

3. **从备份恢复**
   - 打开设置 > 更新 > 备份
   - 选择更新前的备份
   - 点击"恢复"

4. **手动更新**
   - 停止 OpenClaw 服务
   - 手动替换二进制文件
   - 重新启动服务

---

## 诊断系统问题

### 问题：诊断工具无法运行

**症状**：
- 点击"运行诊断"无反应
- 诊断过程中断

**解决方案**：

1. **检查系统权限**
   - 确保有访问系统信息的权限

2. **检查依赖**
   ```bash
   # 确保系统命令可用
   which ps
   which netstat
   which df
   ```

3. **手动诊断**
   ```bash
   # 检查端口占用
   lsof -i :8080

   # 检查磁盘空间
   df -h

   # 检查内存
   free -h
   ```

### 问题：自动修复失败

**症状**：
- 点击"一键修复"后部分问题仍存在
- 提示修复失败

**解决方案**：

1. **查看具体错误**
   - 检查修复失败的详细错误信息

2. **手动修复**
   - 根据诊断结果手动执行修复步骤

3. **重启后重试**
   - 部分修复需要重启才能生效

4. **重置配置**
   - 如果问题持续，考虑重置配置

---

## 性能问题

### 问题：应用响应缓慢

**症状**：
- 界面操作卡顿
- 切换页面延迟明显

**解决方案**：

1. **检查系统资源**
   ```bash
   # 查看 CPU 和内存使用
   # macOS
   top -l 1

   # Linux
   top -bn1

   # Windows
   tasklist
   ```

2. **清理日志缓存**
   ```bash
   # 清理旧日志
   find ~/.openclaw/logs -name "*.log" -mtime +7 -delete
   ```

3. **禁用硬件加速**
   - 在设置中禁用 GPU 加速
   - 重启应用

### 问题：内存占用过高

**症状**：
- 系统提示内存不足
- 应用占用内存持续增长

**解决方案**：

1. **限制日志保留**
   ```yaml
   logging:
     max_size: 100MB
     max_files: 3
   ```

2. **调整缓存设置**
   ```yaml
   cache:
     max_size: 500MB
     ttl: 3600  # 1小时
   ```

3. **定期重启服务**
   - 设置自动重启计划
   - 或使用系统定时任务

---

## 界面问题

### 问题：界面显示异常

**症状**：
- 界面元素错位
- 文字显示不全
- 图片无法加载

**解决方案**：

1. **重置缩放**
   - 按 `Ctrl/Cmd + 0` 恢复默认缩放

2. **清除缓存**
   ```bash
   # 删除前端缓存
   # macOS
   rm -rf ~/Library/Caches/OpenClaw\ Manager

   # Windows
   rmdir /s "%LOCALAPPDATA%\OpenClaw Manager\Cache"

   # Linux
   rm -rf ~/.cache/OpenClaw\ Manager
   ```

3. **更新显卡驱动**
   - 确保显卡驱动是最新版本

### 问题：字体显示模糊

**症状**：
- 界面文字模糊
- 字体渲染不清晰

**解决方案**：

**macOS:**
```bash
# 禁用字体平滑
defaults write -g CGFontRenderingFontSmoothingDisabled -bool NO
```

**Windows:**
1. 显示设置 > 缩放与布局
2. 尝试不同的缩放比例
3. 启用"修复应用缩放"

---

## 日志查看方法

### 应用日志位置

**OpenClaw Manager:**
- **macOS**: `~/Library/Logs/OpenClaw Manager/log.log`
- **Windows**: `%APPDATA%\OpenClaw Manager\logs\log.log`
- **Linux**: `~/.config/OpenClaw Manager/logs/log.log`

**OpenClaw 服务:**
- **macOS**: `~/.openclaw/logs/openclaw.log`
- **Windows**: `%USERPROFILE%\.openclaw\logs\openclaw.log`
- **Linux**: `~/.local/share/openclaw/logs/openclaw.log`

### 查看日志命令

```bash
# 实时查看日志
tail -f ~/.openclaw/logs/openclaw.log

# 查看最后 100 行
tail -n 100 ~/.openclaw/logs/openclaw.log

# 搜索特定错误
grep -i "error" ~/.openclaw/logs/openclaw.log

# 查看特定时间段的日志
sed -n '/2024-03-01 10:00/,/2024-03-01 11:00/p' ~/.openclaw/logs/openclaw.log
```

### 日志级别说明

- **ERROR**: 错误信息，需要关注
- **WARN**: 警告信息，可能需要注意
- **INFO**: 一般信息，正常操作记录
- **DEBUG**: 调试信息，详细的操作记录
- **TRACE**: 最详细的跟踪信息

### 导出日志

```bash
# 打包日志文件用于支持请求
tar -czvf openclaw-logs-$(date +%Y%m%d).tar.gz ~/.openclaw/logs/
```

---

## 重置配置方法

### 方法 1：通过界面重置

1. 打开 OpenClaw Manager
2. 进入 **设置 > 高级**
3. 点击 **"重置所有配置"**
4. 确认重置操作
5. 应用将自动重启

### 方法 2：手动重置

**重置应用配置：**
```bash
# 完全退出应用后执行

# macOS
rm -rf ~/Library/Application\ Support/OpenClaw\ Manager

# Windows
rmdir /s "%APPDATA%\OpenClaw Manager"

# Linux
rm -rf ~/.config/OpenClaw\ Manager
```

**重置 OpenClaw 配置：**
```bash
# 备份配置
cp ~/.openclaw/config.yaml ~/.openclaw/config.yaml.backup.$(date +%Y%m%d)

# 删除配置（会自动生成默认配置）
rm ~/.openclaw/config.yaml

# 或者重置整个 OpenClaw 目录
mv ~/.openclaw ~/.openclaw.backup.$(date +%Y%m%d)
```

### 方法 3：命令行重置

```bash
# 重置为默认配置
openclaw config reset

# 或使用 Manager 命令行
openclaw-manager --reset-config
```

---

## 获取帮助

### 自助资源

1. **查看文档**
   - [用户指南](./USER_GUIDE.md)
   - [常见问题](./FAQ.md)
   - [更新日志](./CHANGELOG.md)

2. **搜索问题**
   - [GitHub Issues](https://github.com/openclaw/openclaw-manager/issues)
   - [GitHub Discussions](https://github.com/openclaw/openclaw-manager/discussions)

3. **生成诊断报告**
   ```bash
   # 生成包含系统信息、配置、日志的诊断包
   openclaw-manager --diagnostic-report
   ```

### 联系支持

如果以上方法无法解决问题，请通过以下方式联系支持：

**GitHub Issues:**
- 访问 [Issues 页面](https://github.com/openclaw/openclaw-manager/issues)
- 使用 [Bug 报告模板](https://github.com/openclaw/openclaw-manager/issues/new?template=bug_report.md)
- 提供以下信息：
  - 操作系统版本
  - OpenClaw Manager 版本
  - 问题描述和重现步骤
  - 相关日志片段
  - 截图（如适用）

**邮件支持:**
- 发送邮件至: support@openclaw.io
- 主题格式: `[Bug] 简短描述` 或 `[Help] 简短描述`
- 附上诊断报告（如有）

**社区支持:**
- 加入 [GitHub Discussions](https://github.com/openclaw/openclaw-manager/discussions)
- 在讨论区提问

### 提交问题前的准备

为了更快地获得帮助，请准备以下信息：

1. **系统信息**
   ```bash
   # macOS
   sw_vers
   uname -a

   # Windows
   systeminfo

   # Linux
   lsb_release -a
   uname -a
   ```

2. **应用版本**
   ```bash
   openclaw-manager --version
   openclaw --version
   ```

3. **问题日志**
   - 导出相关时间段的日志
   - 标注错误发生的时间点

4. **重现步骤**
   - 详细描述导致问题的操作步骤
   - 说明问题是否可以稳定重现

---

## 常见错误速查表

| 错误信息 | 可能原因 | 解决方案 |
|---------|---------|---------|
| "Permission denied" | 权限不足 | 检查并修复文件权限 |
| "Address already in use" | 端口被占用 | 更换端口或终止占用进程 |
| "Connection refused" | 服务未启动或网络问题 | 检查服务状态和网络连接 |
| "Invalid configuration" | 配置格式错误 | 验证 YAML 语法 |
| "API key invalid" | API 密钥错误 | 检查并重新输入 API 密钥 |
| "Out of memory" | 内存不足 | 关闭其他应用或增加内存 |
| "Disk full" | 磁盘空间不足 | 清理磁盘空间 |
| "Timeout" | 连接超时 | 检查网络质量或增加超时时间 |
| "Secure storage error" | 系统密钥存储问题 | 检查钥匙串/凭证管理器 |
| "Installation failed" | 安装脚本失败 | 检查网络或尝试离线安装 |
| "Service already running" | 服务已在运行 | 等待或重启应用 |
| "Health check failed" | 健康检查失败 | 检查服务日志 |
| "Download failed" | 下载失败 | 检查网络或更换镜像源 |
| "Checksum verification failed" | 文件校验失败 | 重新下载或更换源 |
| "Configuration locked" | 配置被锁定 | 检查是否有其他进程占用 |
| "Unsupported platform" | 不支持的系统 | 检查系统要求 |
| "All mirrors failed" | 所有镜像失败 | 检查网络或稍后重试 |
| "Port in use" | 端口被占用 | 更换端口或终止占用进程 |
| "Process crashed" | 进程崩溃 | 查看日志检查原因 |

---

*故障排除指南版本: 1.1.0*
*最后更新: 2026-03-03*
