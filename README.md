<p align="center">
  <img src="./logo.png" width="150">
</p>

This repository contains the source code for the git-icons CLI. [The app can be found over here](https://samddenty.gumroad.com/l/git-icons)

[![Banner](./banner.png)](https://samddenty.gumroad.com/l/git-icons)

## Building locally

```bash
brew install mysql-client
cargo install diesel_cli --no-default-features --features mysql
cargo run -- sync --token INSERT_TOKEN
```
