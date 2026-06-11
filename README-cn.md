# cprust

一个功能丰富的文件和目录复制工具，使用 Rust 编写。支持递归复制、符号链接、元数据保留、进度条、多源复制等功能。

## 特性

- 递归复制文件和目录
- 符号链接支持（作为符号链接复制或跟随链接）
- 保留文件元数据（时间戳、权限）
- 大文件和目录的进度条
- 多源文件复制到单个目标目录
- 使用 `--parents` 重建完整目录结构
- 不覆盖（no-clobber）和强制覆盖模式
- 详细输出和静默输出模式
- 支持相对路径和绝对路径
- 防止将文件或目录复制到自身
- 防止用目录覆盖文件
- 所有失败模式都有清晰的错误信息

## 安装

### 从源码构建

```bash
git clone https://github.com/devpew/cprust.git
cd cprust
cargo build --release
```

生成的可执行文件位于 `target/release/cprust`。

### 全局安装

```bash
cargo install --path .
```

## 用法

```bash
cprust [OPTION]... SOURCE... DESTINATION
```

### 选项

| 标志 | 说明 |
|------|------|
| `-r`, `-R` | 递归复制目录 |
| `-v` | 详细模式 — 打印每个复制的文件 |
| `-q` | 静默模式 — 抑制输出 |
| `-p` | 保留文件元数据（时间戳、权限） |
| `-L` | 跟随符号链接（复制目标，而非链接） |
| `-P` | 不跟随符号链接（作为符号链接复制，默认） |
| `-n` | 不覆盖 — 不覆盖已有文件 |
| `-f` | 强制 — 覆盖已有文件 |
| `--parents` | 重建完整目录结构 |
| `--progress` | 为大文件显示进度条 |
| `-h`, `--help` | 显示帮助信息 |

### 示例

**复制文件：**

```bash
cprust file.txt /tmp/file.txt
```

**将文件复制进目录：**

```bash
cprust file.txt /tmp/
```

**复制多个文件到目录：**

```bash
cprust file1.txt file2.txt dir/
```

**递归复制目录：**

```bash
cprust -r mydir /tmp/
```

**详细输出复制：**

```bash
cprust -rv mydir /tmp/
```

**保留元数据复制：**

```bash
cprust -rp mydir /tmp/
```

**带进度条复制：**

```bash
cprust --progress largefile.iso /backup/
```

**不覆盖已有文件：**

```bash
cprust -n file.txt /tmp/
```

**强制覆盖：**

```bash
cprust -f file.txt /tmp/existing.txt
```

**重建目录结构：**

```bash
cprust --parents a/b/c/file.txt /backup/
# 创建 /backup/a/b/c/file.txt
```

**跟随符号链接：**

```bash
cprust -L link.txt /tmp/
```

**作为符号链接复制：**

```bash
cprust -P link.txt /tmp/
```

## 错误处理

工具在发生错误时返回非零退出码：

| 错误 | 说明 |
|------|------|
| `No such file or directory` | 源文件或源目录不存在 |
| `are the same file` | 源和目标解析为相同的文件 |
| `are the same directory` | 源和目标解析为相同的目录 |
| `cannot overwrite file with directory` | 尝试用目录覆盖已有文件 |
| `cannot copy multiple sources to ... which does not exist` | 多源复制要求目标目录已存在 |
| `omitting directory (use -r for recursive)` | 尝试复制目录但未使用 `-r` 标志 |

## 要求

- Rust 1.75+（2024 年版）

## 测试

```bash
cargo test
```

## CI

本项目使用 GitHub Actions 进行持续集成：

- **Clippy** — 使用 `-D warnings` 进行 lint 检查
- **Tests** — 完整的集成测试套件
- **Format** — `cargo fmt` 风格检查

## 许可证

MIT
