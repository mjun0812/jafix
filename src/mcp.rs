use std::path::{Path, PathBuf};

use rmcp::handler::server::{router::tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{Implementation, ServerCapabilities, ServerConfig};
use rmcp::{Json, ServerHandler, ServiceExt, tool, tool_handler, tool_router, transport};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::apply::TargetLines;
use crate::config::Configs;
use crate::diagnostics::{self, Diagnostic};
use crate::fix_content;

pub fn serve(project_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let dir = project_dir
        .canonicalize()
        .map_err(|error| format!("{}: {error}", project_dir.display()))?;
    if !dir.is_dir() {
        return Err(format!("{}: ディレクトリを指定してください", project_dir.display()).into());
    }
    let server = Proofreader {
        project_dir: dir,
        tool_router: Proofreader::tool_router(),
    };
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            server.serve(transport::stdio()).await?.waiting().await?;
            Ok(())
        })
}

#[derive(Clone)]
struct Proofreader {
    project_dir: PathBuf,
    tool_router: ToolRouter<Self>,
}

#[derive(Deserialize, JsonSchema)]
struct TextInput {
    /// Japanese prose, interpreted as Markdown.
    text: String,
}

#[derive(Serialize, JsonSchema)]
struct CheckedText {
    findings: Vec<Diagnostic>,
}

#[derive(Serialize, JsonSchema)]
struct FixedText {
    text: String,
    /// Findings in the corrected text, using its line and column numbers.
    findings: Vec<Diagnostic>,
}

#[tool_router]
impl Proofreader {
    #[tool(
        description = "Check Japanese prose before writing it to a file. Return rule IDs and locations.",
        annotations(read_only_hint = true)
    )]
    fn check_text(
        &self,
        Parameters(TextInput { text }): Parameters<TextInput>,
    ) -> Result<Json<CheckedText>, String> {
        let mut configs = Configs::load()?;
        let config = configs.for_directory(&self.project_dir)?;
        let findings = diagnostics::check(&text, config, &TargetLines::All, None, false);
        Ok(Json(CheckedText { findings }))
    }

    #[tool(
        description = "Return automatically corrected Japanese prose and its remaining findings. The caller can use the returned text.",
        annotations(read_only_hint = true)
    )]
    fn fix_text(
        &self,
        Parameters(TextInput { text }): Parameters<TextInput>,
    ) -> Result<Json<FixedText>, String> {
        let mut configs = Configs::load()?;
        let config = configs.for_directory(&self.project_dir)?;
        let targets = TargetLines::All;
        let fixed = fix_content(&text, &targets, config);
        let findings = diagnostics::check(&fixed.content, config, &targets, None, false);
        Ok(Json(FixedText {
            text: fixed.content,
            findings,
        }))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Proofreader {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("jafix", env!("CARGO_PKG_VERSION")))
    }
}
