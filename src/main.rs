//! qs_notary: post-quantum SBOM notary CLI (Dilithium5 sign/verify).

mod crypto;
mod key_provider;
mod ledger;
mod policy;
mod sign;
mod sign_all;
mod verify;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "qs_notary")]
#[command(about = "Post-quantum SBOM notary with Dilithium5 signing")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Dilithium5 keypair (public.key and private.key).
    #[command(name = "generate-keys")]
    GenerateKeys {
        /// Directory to write public.key and private.key (default: current directory).
        #[arg(long)]
        output_dir: Option<PathBuf>,
    },

    /// Sign an SBOM file (CycloneDX or SPDX JSON). Writes .sig and appends to ledger.
    Sign {
        /// Path to the SBOM file.
        #[arg(value_name = "SBOM")]
        sbom_path: PathBuf,

        /// Path to the private key file (ignored if --kms is set).
        #[arg(value_name = "PRIVATE_KEY", short, long)]
        private_key: PathBuf,

        /// Use mock KMS instead of local key file (test only).
        #[arg(long)]
        kms: bool,

        /// Path to the ledger file (default: ledger.json in current directory).
        #[arg(long, default_value = "ledger.json")]
        ledger: PathBuf,

        /// URL of transparency log server (e.g. http://localhost:8080); uploads entry in background.
        #[arg(long)]
        server_url: Option<String>,
    },

    /// Verify an SBOM file against a signature and public key.
    Verify {
        /// Path to the original SBOM file.
        #[arg(value_name = "SBOM")]
        sbom_path: PathBuf,

        /// Path to the signature file.
        #[arg(value_name = "SIGNATURE")]
        signature_path: PathBuf,

        /// Path to the public key file.
        #[arg(value_name = "PUBLIC_KEY", short, long)]
        public_key: PathBuf,

        /// Path to policy JSON (optional; enforces allowlist and max_age when set).
        #[arg(long)]
        policy: Option<PathBuf>,
    },

    /// Recursively sign all files in a directory, then create and sign manifest.json.
    #[command(name = "sign-all")]
    SignAll {
        /// Directory to sign (recursive).
        #[arg(value_name = "DIR")]
        dir: PathBuf,

        /// Path to the private key file (ignored if --kms is set).
        #[arg(value_name = "PRIVATE_KEY", short, long)]
        private_key: PathBuf,

        /// Use mock KMS instead of local key file (test only).
        #[arg(long)]
        kms: bool,

        /// Path to the ledger file (default: ledger.json).
        #[arg(long, default_value = "ledger.json")]
        ledger: PathBuf,

        /// URL of transparency log server; uploads each entry in background.
        #[arg(long)]
        server_url: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::GenerateKeys { output_dir } => {
            let dir = output_dir.unwrap_or_else(|| PathBuf::from("."));
            crypto::generate_keypair(&dir)?;
            println!("Keys written to {} (public.key, private.key)", dir.display());
        }
        Commands::Sign {
            sbom_path,
            private_key,
            kms,
            ledger,
            server_url,
        } => {
            let provider: Box<dyn key_provider::KeyProvider> = if kms {
                Box::new(key_provider::MockKmsProvider::new())
            } else {
                Box::new(key_provider::FileSystemProvider::new(&private_key))
            };
            sign::run(
                &sbom_path,
                provider.as_ref(),
                &ledger,
                server_url.as_deref(),
            )?;
            println!("Signed and ledger updated.");
        }
        Commands::Verify {
            sbom_path,
            signature_path,
            public_key,
            policy,
        } => {
            verify::run(
                &sbom_path,
                &signature_path,
                &public_key,
                policy.as_deref(),
            )?;
        }
        Commands::SignAll {
            dir,
            private_key,
            kms,
            ledger,
            server_url,
        } => {
            let provider: Box<dyn key_provider::KeyProvider> = if kms {
                Box::new(key_provider::MockKmsProvider::new())
            } else {
                Box::new(key_provider::FileSystemProvider::new(&private_key))
            };
            sign_all::run(&dir, provider.as_ref(), &ledger, server_url.as_deref())?;
            println!("Signed all files and manifest.");
        }
    }
    Ok(())
}
