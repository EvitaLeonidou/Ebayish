{
  description = "Full-stack development environment for Rust + React application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05"; # Stable channel with binaries
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain and tools
            bunyan-rs
            rustc
            cargo
            rustfmt
            rust-analyzer
            clippy

            # Database tools
            postgresql_16
            sqlx-cli

            # Node.js ecosystem and package managers
            nodejs_20
            nodePackages.npm
            yarn # Alternative package manager

            # Frontend development tools (will be installed via npm)
            typescript

            # SSL/TLS certificates and security
            openssl
            mkcert

            # Container and deployment tools
            docker
            docker-compose

            # Development utilities
            curl
            jq
            git
            act

            # report dependenciesa
            typst

            # Build dependencies and system libraries
            pkg-config
            libiconv

          ];

          # Rust environment variables
          RUST_LOG = "debug";
          RUST_BACKTRACE = "1";
          CARGO_NET_GIT_FETCH_WITH_CLI = "true";
          SQLX_OFFLINE = "false";

          # PostgreSQL environment for local development
          POSTGRES_USER = "postgres";
          POSTGRES_PASSWORD = "password";
          POSTGRES_DB = "db";
          POSTGRES_PORT = "5432";
          DATABASE_URL = "postgres://postgres:password@localhost:5432/db";

          # Frontend development environment
          VITE_API_BASE_URL = "https://localhost:6000";
          NODE_ENV = "development";

          # Build tool configurations
          ESLINT_USE_FLAT_CONFIG = "true";

          shellHook = ''

            # Set up local npm directories
            mkdir -p .npm-global
            mkdir -p .npm-cache

            # Configure npm via environment variables (avoids permission issues)
            export NPM_CONFIG_PREFIX="$PWD/.npm-global"
            export NPM_CONFIG_CACHE="$PWD/.npm-cache"
            export NPM_CONFIG_USERCONFIG="$PWD/.npmrc"
            export PATH="$PWD/.npm-global/bin:$PATH"

            # Create local .npmrc if it doesn't exist
            if [ ! -f .npmrc ]; then
              echo "prefix=$PWD/.npm-global" > .npmrc
              echo "cache=$PWD/.npm-cache" >> .npmrc
            fi

            # Switch to fish if available (AFTER setup)
            if command -v fish &> /dev/null; then
              exec fish
            fi
          '';
        };
      }
    );
}
