use sc_cli::CliConfiguration;

/// [`SimnodeCli`] implementation
pub struct MoonbeamCli;

impl substrate_simnode::SimnodeCli for MoonbeamCli {
	type CliConfig = sc_cli::RunCmd;
	type SubstrateCli = moonbeam_cli::Cli;

	fn cli_config(cli: &Self::SubstrateCli) -> &Self::CliConfig {
		&cli.run.base.base
	}

	fn log_filters(cli_config: &Self::CliConfig) -> Result<String, sc_cli::Error> {
		cli_config.log_filters()
	}
}
