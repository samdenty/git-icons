<h1 align="center">
  <img src="./logo.png" width="150">
</h1>

## [Get the Mac App](https://samddenty.gumroad.com/l/git-icons)

Automatically adds project logos to your locally cloned GitHub repos. [Youtube Video](https://www.youtube.com/watch?v=jrO3qSEpAFU)

This repository contains the source code for the git-icons CLI. You can [sponsor this project](https://github.com/sponsors/samdenty)

## Running the CLI

You can run the CLI using the below commands, but you should [get the application for more features](https://samddenty.gumroad.com/l/git-icons)

<!-- brew install mysql-client
cargo install diesel_cli --no-default-features --features mysql -->

```bash
# To install rust
curl https://sh.rustup.rs -sSf | sh

# Clone the repository
git clone https://github.com/samdenty/git-icons
cd git-icons

cargo run -- sync
# or with github token (for private repos)
cargo run -- sync --token INSERT_TOKEN
```

[![Banner](./banner.gif)](https://samddenty.gumroad.com/l/git-icons)
