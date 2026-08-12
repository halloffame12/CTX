use crate::commands::Project;
use crate::errors::CtxResult;
use crate::mcp::server;
use crate::mcp::tools::McpEnv;

pub fn cmd_mcp(project: &Project, root_override: Option<&std::path::Path>) -> CtxResult<()> {
    let root = root_override
        .map(|r| r.to_path_buf())
        .unwrap_or_else(|| project.root.clone());
    let project = if root != project.root {
        Project::open(&root, Some(&root))?
    } else {
        project.clone()
    };
    let env = McpEnv { project };
    server::run(&env)
}
