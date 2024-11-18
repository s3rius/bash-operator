#[derive(Debug, Clone, clap::Parser)]
pub struct Cli {
    /// Log level.
    #[arg(short, long, default_value = "info", env = "BASH_OPERATOR_LOG_LEVEL")]
    pub log_level: tracing::level_filters::LevelFilter,

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
    #[arg(long, env = "BASH_OPERATOR_NAMESPACE")]
    pub namespace: Option<String>,

    /// Watch all namespaces
    /// If this flag is set, the operator will watch
    /// all namespaces.
    /// If both `namespace` and `all_namespaces` are set,
    /// `namespace` will be ignored.
    #[arg(long, default_value = "false", env = "BASH_OPERATOR_ALL_NAMESPACES")]
    pub all_namespaces: bool,
}
