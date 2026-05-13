mod executor;
mod tools;

use std::sync::Arc;
use openai::Client;
use executor::ModbusExecutor;
use tools::modbus_tools;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("MINIMAX_API_KEY")
        .expect("MINIMAX_API_KEY environment variable not set");

    let client = Client::new(&api_key)?;
    let tools = modbus_tools();
    let executor = Arc::new(ModbusExecutor::new());

    println!("=== Modbus Agent ===");
    println!("Initial state: {}\n", executor.show_status());

    // Test: add coil
    println!("--- Test: Add Coil at address 0 with value true ---");
    let response = client
        .chat_with_executor(tools.clone(), executor.clone())
        .model("MiniMax-M2.7-highspeed")
        .messages(vec![
            openai::Message::system(
                "你是一个Modbus设备控制器。你可以添加或删除线圈、离散输入和寄存器。\
                 当用户请求操作时，使用对应的工具完成操作并返回结果。\
                 只在用户明确要求时才执行操作，不要自动执行。\
                 操作完成后显示设备状态。"
            ),
            openai::Message::user("在地址0开始添加3个值为true的线圈"),
        ])
        .send()?;

    println!("Response: {:?}", response.choices.first().map(|c| &c.message.content));
    println!("State: {}\n", executor.show_status());

    // Test: add register
    println!("--- Test: Add Register at address 10 with value 123 ---");
    let response = client
        .chat_with_executor(tools.clone(), executor.clone())
        .model("MiniMax-M2.7-highspeed")
        .messages(vec![
            openai::Message::system(
                "你是一个Modbus设备控制器。你可以添加或删除线圈、离散输入和寄存器。\
                 当用户请求操作时，使用对应的工具完成操作并返回结果。\
                 只在用户明确要求时才执行操作，不要自动执行。\
                 操作完成后显示设备状态。"
            ),
            openai::Message::user("在地址10添加4个值为123的寄存器"),
        ])
        .send()?;

    println!("Response: {:?}", response.choices.first().map(|c| &c.message.content));
    println!("State: {}\n", executor.show_status());

    // Test: add discrete input
    println!("--- Test: Add Discrete Input at address 5 with value false ---");
    let response = client
        .chat_with_executor(tools.clone(), executor.clone())
        .model("MiniMax-M2.7-highspeed")
        .messages(vec![
            openai::Message::system(
                "你是一个Modbus设备控制器。你可以添加或删除线圈、离散输入和寄存器。\
                 当用户请求操作时，使用对应的工具完成操作并返回结果。\
                 只在用户明确要求时才执行操作，不要自动执行。\
                 操作完成后显示设备状态。"
            ),
            openai::Message::user("在地址5添加4个值为false的离散输入"),
        ])
        .send()?;

    println!("Response: {:?}", response.choices.first().map(|c| &c.message.content));
    println!("State: {}\n", executor.show_status());


    println!("=== Begin do delete ===\n");

    // Test: delete coil
    println!("--- Test: Delete Coil at address 0 ---");
    let response = client
        .chat_with_executor(tools.clone(), executor.clone())
        .model("MiniMax-M2.7-highspeed")
        .messages(vec![
            openai::Message::system(
                "你是一个Modbus设备控制器。你可以添加或删除线圈、离散输入和寄存器。\
                 当用户请求操作时，使用对应的工具完成操作并返回结果。\
                 只在用户明确要求时才执行操作，不要自动执行。\
                 操作完成后显示设备状态。"
            ),
            openai::Message::user("删除地址0的线圈"),
        ])
        .send()?;

    println!("Response: {:?}", response.choices.first().map(|c| &c.message.content));
    println!("State: {}\n", executor.show_status());

    // Test: delete register
    println!("--- Test: Delete Register at address 10 ---");
    let response = client
        .chat_with_executor(tools.clone(), executor.clone())
        .model("MiniMax-M2.7-highspeed")
        .messages(vec![
            openai::Message::system(
                "你是一个Modbus设备控制器。你可以添加或删除线圈、离散输入和寄存器。\
                 当用户请求操作时，使用对应的工具完成操作并返回结果。\
                 只在用户明确要求时才执行操作，不要自动执行。\
                 操作完成后显示设备状态。"
            ),
            openai::Message::user("删除地址10的寄存器"),
        ])
        .send()?;

    println!("Response: {:?}", response.choices.first().map(|c| &c.message.content));
    println!("State: {}\n", executor.show_status());

    // Test: delete discrete input
    println!("--- Test: Delete Discrete Input at address 5 ---");
    let response = client
        .chat_with_executor(tools.clone(), executor.clone())
        .model("MiniMax-M2.7-highspeed")
        .messages(vec![
            openai::Message::system(
                "你是一个Modbus设备控制器。你可以添加或删除线圈、离散输入和寄存器。\
                 当用户请求操作时，使用对应的工具完成操作并返回结果。\
                 只在用户明确要求时才执行操作，不要自动执行。\
                 操作完成后显示设备状态。"
            ),
            openai::Message::user("删除地址5的离散输入"),
        ])
        .send()?;

    println!("Response: {:?}", response.choices.first().map(|c| &c.message.content));
    println!("Final State: {}\n", executor.show_status());

    println!("=== All tests completed ===");
    Ok(())
}
