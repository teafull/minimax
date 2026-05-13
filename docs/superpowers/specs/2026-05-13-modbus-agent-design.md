# modbusAgent 设计规格

## 概述

创建 `modbusAgent` 主程序，验证 OpenAI 工具执行机制是否支持 Modbus 操作的添加和删除。

## 功能需求

1. 定义 6 个 Modbus 工具函数（添加/删除线圈、离散输入、寄存器）
2. 实现 `ToolExecutor` trait 执行模拟的 Modbus 操作
3. 使用 `chat_with_executor()` 验证工具调用流程

## 新增文件

```
crates/
  modbus_agent/
    Cargo.toml
    src/
      main.rs        # 主程序入口
      executor.rs    # ToolExecutor 实现
      tools.rs       # 工具定义
```

## 工具定义 (tools.rs)

| 工具名 | 参数 | 返回值 |
|--------|------|--------|
| `add_coil` | `address: i32, value: bool` | `{"status": "ok", "address": xxx}` |
| `delete_coil` | `address: i32` | `{"status": "ok", "address": xxx}` |
| `add_discrete_input` | `address: i32, value: bool` | `{"status": "ok", "address": xxx}` |
| `delete_discrete_input` | `address: i32` | `{"status": "ok", "address": xxx}` |
| `add_register` | `address: i32, value: i32` | `{"status": "ok", "address": xxx}` |
| `delete_register` | `address: i32` | `{"status": "ok", "address": xxx}` |

## ToolExecutor 实现 (executor.rs)

- 维护模拟存储：`coils: HashMap<i32, bool>`, `discrete_inputs: HashMap<i32, bool>`, `registers: HashMap<i32, i32>`
- 实现 `execute(tool_name, arguments)` 方法
- 根据 tool_name 分发到对应的 Modbus 操作
- 操作成功返回 JSON 字符串，失败返回错误

## 主程序 (main.rs)

1. 从环境变量 `MINIMAX_API_KEY` 读取 API key
2. 创建 `Client`
3. 定义 6 个 `Tool` 结构
4. 创建自定义 executor 实例
5. 使用 `chat_with_executor()` 构建请求
6. 设置 system prompt 说明 Modbus 工具用法
7. 发送测试消息触发工具调用
8. 打印最终响应

## 依赖

- `minimax-openai` crate (workspace 内部依赖)
- `serde_json`
- `std::collections::HashMap`

## 测试场景

验证 LLM 能够：
1. 理解添加/删除请求
2. 调用正确的工具
3. 获得执行结果
