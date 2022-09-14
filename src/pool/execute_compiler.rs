use super::Method;

pub struct ExecuteCompiler;

impl ExecuteCompiler {
    pub fn compile(execute: Vec<Method>) -> String {
        let mut code = String::new();
    
        let method_count = execute.len();
        
        for (index, method) in execute.into_iter().enumerate() {
            code.push_str(
                format!(
                    "var result{index} = API.{}({});",
                    method.name,
                    serde_json::to_string(&method.params).unwrap()
                )
                .as_str(),
            );
        }
    
        code.push_str("return [");
    
        for i in 0..method_count {
            code.push_str(format!("result{i},").as_str());
        }
    
        code.push_str("];");
    
        code
    }
}
