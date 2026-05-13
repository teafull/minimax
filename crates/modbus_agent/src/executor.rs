use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use openai::ToolExecutor;

pub struct ModbusExecutor {
    coils: Arc<Mutex<HashMap<i32, bool>>>,
    discrete_inputs: Arc<Mutex<HashMap<i32, bool>>>,
    registers: Arc<Mutex<HashMap<i32, i32>>>,
}

impl ModbusExecutor {
    pub fn new() -> Self {
        Self {
            coils: Arc::new(Mutex::new(HashMap::new())),
            discrete_inputs: Arc::new(Mutex::new(HashMap::new())),
            registers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn show_status(&self) -> String {
        let coils = self.coils.lock().unwrap();
        let discrete_inputs = self.discrete_inputs.lock().unwrap();
        let registers = self.registers.lock().unwrap();

        format!(
            "Coils: {:?}, Discrete Inputs: {:?}, Registers: {:?}",
            *coils, *discrete_inputs, *registers
        )
    }
}

impl ToolExecutor for ModbusExecutor {
    fn execute(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match tool_name {
            "add_coil" => {
                let address = arguments["address"].as_i64().unwrap() as i32;
                let value = arguments["value"].as_bool().unwrap();
                self.coils.lock().unwrap().insert(address, value);
                Ok(serde_json::json!({
                    "status": "ok",
                    "operation": "add_coil",
                    "address": address,
                    "value": value
                }).to_string())
            }
            "delete_coil" => {
                let address = arguments["address"].as_i64().unwrap() as i32;
                self.coils.lock().unwrap().remove(&address);
                Ok(serde_json::json!({
                    "status": "ok",
                    "operation": "delete_coil",
                    "address": address
                }).to_string())
            }
            "add_discrete_input" => {
                let address = arguments["address"].as_i64().unwrap() as i32;
                let value = arguments["value"].as_bool().unwrap();
                self.discrete_inputs.lock().unwrap().insert(address, value);
                Ok(serde_json::json!({
                    "status": "ok",
                    "operation": "add_discrete_input",
                    "address": address,
                    "value": value
                }).to_string())
            }
            "delete_discrete_input" => {
                let address = arguments["address"].as_i64().unwrap() as i32;
                self.discrete_inputs.lock().unwrap().remove(&address);
                Ok(serde_json::json!({
                    "status": "ok",
                    "operation": "delete_discrete_input",
                    "address": address
                }).to_string())
            }
            "add_register" => {
                let address = arguments["address"].as_i64().unwrap() as i32;
                let value = arguments["value"].as_i64().unwrap() as i32;
                self.registers.lock().unwrap().insert(address, value);
                Ok(serde_json::json!({
                    "status": "ok",
                    "operation": "add_register",
                    "address": address,
                    "value": value
                }).to_string())
            }
            "delete_register" => {
                let address = arguments["address"].as_i64().unwrap() as i32;
                self.registers.lock().unwrap().remove(&address);
                Ok(serde_json::json!({
                    "status": "ok",
                    "operation": "delete_register",
                    "address": address
                }).to_string())
            }
            _ => Err(format!("Unknown tool: {}", tool_name).into()),
        }
    }
}
