use openai::{FunctionDefinition, Tool};

pub fn modbus_tools() -> Vec<Tool> {
    vec![
        Tool {
            type_: "function".to_string(),
            function: FunctionDefinition {
                name: "add_coil".to_string(),
                description: "添加一个Modbus线圈".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "address": {
                            "type": "integer",
                            "description": "线圈地址"
                        },
                        "value": {
                            "type": "boolean",
                            "description": "线圈值 (true=1, false=0)"
                        }
                    },
                    "required": ["address", "value"]
                }),
            },
        },
        Tool {
            type_: "function".to_string(),
            function: FunctionDefinition {
                name: "delete_coil".to_string(),
                description: "删除一个Modbus线圈".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "address": {
                            "type": "integer",
                            "description": "线圈地址"
                        }
                    },
                    "required": ["address"]
                }),
            },
        },
        Tool {
            type_: "function".to_string(),
            function: FunctionDefinition {
                name: "add_discrete_input".to_string(),
                description: "添加一个Modbus离散输入".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "address": {
                            "type": "integer",
                            "description": "离散输入地址"
                        },
                        "value": {
                            "type": "boolean",
                            "description": "离散输入值 (true=1, false=0)"
                        }
                    },
                    "required": ["address", "value"]
                }),
            },
        },
        Tool {
            type_: "function".to_string(),
            function: FunctionDefinition {
                name: "delete_discrete_input".to_string(),
                description: "删除一个Modbus离散输入".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "address": {
                            "type": "integer",
                            "description": "离散输入地址"
                        }
                    },
                    "required": ["address"]
                }),
            },
        },
        Tool {
            type_: "function".to_string(),
            function: FunctionDefinition {
                name: "add_register".to_string(),
                description: "添加一个Modbus保持寄存器".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "address": {
                            "type": "integer",
                            "description": "寄存器地址"
                        },
                        "value": {
                            "type": "integer",
                            "description": "寄存器值 (0-65535)"
                        }
                    },
                    "required": ["address", "value"]
                }),
            },
        },
        Tool {
            type_: "function".to_string(),
            function: FunctionDefinition {
                name: "delete_register".to_string(),
                description: "删除一个Modbus保持寄存器".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "address": {
                            "type": "integer",
                            "description": "寄存器地址"
                        }
                    },
                    "required": ["address"]
                }),
            },
        },
    ]
}
