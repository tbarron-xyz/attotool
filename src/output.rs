pub trait Output: Send {
    fn print_tool_call(&mut self, tool: &str, primary_value: &str);
    fn print_result(&mut self, tool: &str, _primary_value: &str, result: &str);
    fn print_summary(&mut self, tool_calls: &[(String, String)]);
    fn print_failure(
        &mut self,
        tool: &str,
        primary_value: &str,
        args_str: &str,
        error: &str,
    );
}

pub struct StdoutOutput {
    pub tool_call_details: bool,
}

impl Output for StdoutOutput {
    fn print_tool_call(&mut self, tool: &str, primary_value: &str) {
        println!("--- [{} {}]", tool, primary_value);
    }

    fn print_result(&mut self, tool: &str, _primary_value: &str, result: &str) {
        if self.tool_call_details {
            let prefixed = format!("[{} {}]\n{}", tool, _primary_value, result);
            println!(
                "Tool call result: {}",
                prefixed.chars().take(500).collect::<String>()
            );
        }
    }

    fn print_summary(&mut self, tool_calls: &[(String, String)]) {
        println!("--- Task tool usage summary");
        for (tool, arg) in tool_calls {
            println!("[{} {}]", tool, arg);
        }
    }

    fn print_failure(
        &mut self,
        tool: &str,
        primary_value: &str,
        _args_str: &str,
        error: &str,
    ) {
        println!("--- [{} {}]", tool, primary_value);
        if self.tool_call_details {
            println!("Tool call failed: [FAILURE {} {}]", tool, _args_str);
            println!("Error: {}", error);
        }
    }
}

pub struct TuiOutput {
    pub sender: tokio::sync::mpsc::Sender<String>,
}

impl Output for TuiOutput {
    fn print_tool_call(&mut self, tool: &str, primary_value: &str) {
        let _ =
            self.sender.try_send(format!("--- [{} {}]", tool, primary_value));
    }

    fn print_result(&mut self, tool: &str, _primary_value: &str, result: &str) {
        if tool == "describe_to_user"
            || tool == "finish_task"
            || tool == "finish_planning"
        {
            let _ = self.sender.try_send(result.to_string());
        }
    }

    fn print_summary(&mut self, tool_calls: &[(String, String)]) {
        let _ = self.sender.try_send("--- Task tool usage summary".to_string());
        for (tool, arg) in tool_calls {
            let _ = self.sender.try_send(format!("[{} {}]", tool, arg));
        }
    }

    fn print_failure(
        &mut self,
        tool: &str,
        primary_value: &str,
        _args_str: &str,
        _error: &str,
    ) {
        let _ =
            self.sender.try_send(format!("--- [{} {}]", tool, primary_value));
    }
}
