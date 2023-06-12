# clir
Clir is a small CLI app written in Rust that helps you keep your filesystem clean by allowing you to define custom global glob patterns to identify and remove unwanted files.

## Features
 - [x] Define custom global glob patterns to identify and remove unwanted files
 - [x] Minimalistic, formatted output for reporting disk usage of specified patterns
 - [x] Supported on all Unix platforms
 
## Installation
### Install from source
(*requires rustc >= 1.56.0*)

```shell
cargo install --path .
```

Make sure that `${HOME}/.cargo/bin` is in your path!

## Usage
To use clir, simply run the `clir` command followed by any additional arguments or options. Here are some examples:

Add a new pattern:
```shell
clir add <pattern>
```

Print a report on currently defined patterns:
```shell
clir
```

Remove files associated with defined patterns:
```shell
clir -r
```

For a comprehensive list of all capabilities and options please run `clir --help`.

## Contributing
Contributions are always welcome. For small changes feel free to submit a PR. For larger changes please create an issue first to discuss your proposal.

## License
Clir is licensed under the [MIT license](https://github.com/heat1q/clir/blob/master/LICENSE).
