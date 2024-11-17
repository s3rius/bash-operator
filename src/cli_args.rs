use std::{path::PathBuf, str::FromStr};

use tracing::level_filters::LevelFilter;

#[derive(Debug, Clone)]
pub enum FileFormat {
    Json,
    Yaml,
}

impl FromStr for FileFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(FileFormat::Json),
            "yaml" => Ok(FileFormat::Yaml),
            _ => Err("Invalid file format".to_string()),
        }
    }
}

#[derive(Debug, Clone, clap::Parser)]
pub struct OperatorArgs {
    /// The group version kind of the resource you want to watch
    /// this string should be in the format `group/version/kind`
    /// e.g. `apps/v1/Deployment`
    /// For apps with no group, omit the group part
    /// e.g. `v1/Secret`
    pub gvk: String,

    /// The function to call on each reconcile
    /// the function should be an exported bash function
    ///
    /// Each function should take 2 arguments:
    /// 1. Current action
    /// 2. The file where full object manifest is stored.
    pub func_name: String,

    /// The namespace to watch
    /// If not provided, the operator will watch
    /// default namespace
    #[arg(long)]
    pub namespace: Option<String>,

    /// Watch all namespaces
    /// If this flag is set, the operator will watch
    /// all namespaces.
    /// If both `namespace` and `all_namespaces` are set,
    /// `namespace` will be ignored.
    #[arg(long, default_value = "false")]
    pub all_namespaces: bool,

    /// Manifest file format
    /// This format will be used to store the object
    /// manifest in a temporary file before calling the
    /// function.
    #[arg(long, default_value = "json")]
    pub file_format: FileFormat,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct Cli {
    /// Log level.
    #[arg(short, long, default_value = "info")]
    pub log_level: LevelFilter,

    #[clap(subcommand)]
    pub subcommand: Cmds,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum UtilsSub {
    AddFinalizer {
        /// Name of the finalizer to add.
        finalizer_name: String,
        /// Path to the manifest of an object to update.
        path_to_mainfest: PathBuf,
    },
    RemoveFinalizer {
        /// Name of the finalizer to remove.
        finalizer_name: String,
        /// Path to the manifest of an object to update.
        path_to_mainfest: PathBuf,
    },
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Cmds {
    /// Run the operator. This command starts the operator and watches the specified resource.
    /// The operator will call the specified function on each reconcile.
    Operator(OperatorArgs),
    /// Utility commands.
    Utils {
        #[clap(subcommand)]
        subcommand: UtilsSub,
    },
}
