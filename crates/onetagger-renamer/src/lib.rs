#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;

use std::path::{Path, PathBuf};
use std::error::Error;
use onetagger_autotag::AudioFileInfoImpl;
use onetagger_tagger::AudioFileInfo;
use serde::{Serialize, Deserialize};

pub mod ac;
pub mod docs;
pub mod parser;

// Re-export
pub use parser::{TemplateParser, SyntaxData, SyntaxType};


/// Renamer itself
pub struct Renamer {
    template: TemplateParser
}

impl Renamer {
    /// Create new instance
    pub fn new(template: TemplateParser) -> Renamer {
        Renamer { template }
    }

    /// Generate new filename
    pub fn generate_name(&mut self, output_dir: impl AsRef<Path>, info: &AudioFileInfo, config: &RenamerConfig) -> PathBuf {
        let mut name = self.template.evaluate(info, config);
        while name.starts_with("/") {
            name = name[1..].to_string()
        }
        if cfg!(windows) {
            name = name.replace("/", "\\");
        }
        let ext = info.path.split(".").last().unwrap_or("");
        output_dir.as_ref().join(format!("{name}.{ext}"))
    }

    /// Generate names - output: [(from, to),...]
    pub fn generate(&mut self, config: &RenamerConfig, limit: usize) -> Result<Vec<(String, PathBuf)>, Box<dyn Error>> {
        if !Path::new(&config.path).exists() {
            return Err("Invalid path!".into());
        }
        
        let out_dir = config.out_dir.clone().unwrap_or(config.path.to_string());
        let files = AudioFileInfo::get_file_list(&config.path, config.subfolders);
        let mut output = vec![];
        for (i, file) in files.iter().enumerate() {
            let info = match AudioFileInfo::load_file(&file, None, None) {
                Ok(info) => info,
                Err(e) => {
                    warn!("Failed loading: {file}. Skipping! {e}");
                    continue;
                }
            };
            let new_name = self.generate_name(&out_dir, &info, config);
            output.push((file.to_string(), new_name));
            if limit != 0 && i >= limit {
                break
            }
        }
        Ok(output)
    }

    /// Rename files
    pub fn rename(&mut self, config: &RenamerConfig) -> Result<(), Box<dyn Error>> {
        let files = self.generate(config, 0)?;
        for (from, to) in files {
            // Don't overwrite
            if !config.overwrite && to.exists() {
                info!("File exists, skipping: {to:?}");
                continue;
            }

            // Create dir
            if let Some(parent) = to.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    warn!("Failed creating dir {parent:?}: {e}");
                }
            }
            // Copy mode
            if config.copy {
                match std::fs::copy(&from, &to) {
                    Ok(_) => info!("Copied: {to:?}"),
                    Err(e) => error!("Failed copying {from} -> {to:?}: {e}"),
                }
            // Move
            } else {
                if std::fs::rename(&from, &to).is_err() {
                    info!("Renaming failed, might be different FS, trying to copy.");
                    match std::fs::copy(&from, &to) {
                        Ok(_) => {
                            info!("Copied: {to:?}");
                            if let Err(e) = std::fs::remove_file(&from) {
                                warn!("Failed deleting {from}: {e}");
                            }
                        },
                        Err(e) => error!("Failed copying {from} -> {to:?}: {e}"),
                    }
                } else {
                    info!("Renamed: {to:?}");
                }
            }
        }

        Ok(())
    }

    /// Generate html from the syntax highlighting
    pub fn generate_html(&self, input: &str) -> String {
        // class prefix
        let prefix = "__renamer_";
        
        let mut output = String::new();
        for syntax in &self.template.syntax {
            let text = input.chars().skip(syntax.start).take(syntax.length).collect::<String>().replace(" ", "&nbsp;");
            let class = match syntax.syntax {
                SyntaxType::Text => "syntax_text",
                SyntaxType::String => "syntax_string",
                SyntaxType::Number => "syntax_number",
                SyntaxType::Function => "syntax_function",
                SyntaxType::Operator => "syntax_operator",
                SyntaxType::Property => "syntax_property",
                SyntaxType::Variable => "syntax_variable",
            };
            output.push_str(&format!("<span class=\"{prefix}{class}\">{text}</span>"));
        }
        output
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenamerConfig {
    pub path: String,
    pub out_dir: Option<String>,
    pub template: String,
    pub copy: bool,
    pub subfolders: bool,
    pub overwrite: bool,
    pub separator: String,
}

/// HTML generation test
#[test]
fn generate_html() {
    let items = [
        "%artist% - %title%",
        "%track%. %artist% - %title%",
        "%artist% - %title% - %bpm% - %key%",
        "%artist% - %album%/%track% - %title%",
        "%year% - %album%/%track% - %artist% - %title%"
    ];
    for i in items {
        let renamer = Renamer::new(TemplateParser::parse(i));
        let output = renamer.generate_html(i);
        println!("{output}")
    }
}
