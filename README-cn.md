# cprust

一个用 Rust 编写的精简文件和目录复制工具。支持递归目录复制、相对与绝对路径、自动目标检测，以及防止复制到相同文件/目录的保护机制。

## 特性

- 递归复制文件和目录
- 支持 `-r` / `-R` 标志以递归复制目录
- 自动检测目标目录 — 如果目标是已存在的目录，则会追加源的名称
- 支持相对路径和绝对路径
- 防止将文件或目录复制到自身
- 防止用目录覆盖文件
- 对缺失文件、源/目标相同以及无效操作给出清晰的错误信息

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
cprust [-r] <source> <destination>
```

### 选项

| 标志 | 说明 |
|------|------|
| `-r`, `-R` | 递归复制目录（复制目录时需使用） |

### 示例

**将文件复制到新位置：**

```bash
cprust file.txt /tmp/file.txt
```

**将文件复制进目录：**

```bash
cprust file.txt /tmp/
```

**递归复制目录：**

```bash
cprust -r mydir /tmp/
```

**将目录复制到新位置：**

```bash
cprust -R ./source_dir ./backup_dir
```

**使用相对路径复制：**

```bash
cprust file.txt ./backup/
```

**使用绝对路径复制：**

```bash
cprust /home/user/docs/file.txt /var/backups/file.txt
```

## 错误处理

工具在发生错误时返回非零退出码：

| 错误 | 说明 |
|------|------|
| `No such file or directory` | 源文件或源目录不存在 |
| `are the same file` | 源和目标解析为相同的文件 |
| `are the same directory` | 源和目标解析为相同的目录 |
| `cannot overwrite file with directory` | 尝试用目录覆盖已有文件 |

## 要求

- Rust 1.75+（2024 年版）

## 许可证

MIT
