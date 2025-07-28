
<img src="./zap.svg" width="70%" alt="a cat's paw struck by lightning from the letters `zap`" />

![latest](https://img.shields.io/github/v/tag/kolja/zap)
[![build](https://github.com/kolja/zap/actions/workflows/rust.yml/badge.svg)](https://github.com/kolja/zap/actions)
[![Coverage Status](https://coveralls.io/repos/github/kolja/zap/badge.svg?branch=main)](https://coveralls.io/github/kolja/zap?branch=main)
[![dependency status](https://deps.rs/repo/github/kolja/zap/status.svg?path=%2F)](https://deps.rs/repo/github/kolja/zap?path=%2F)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

## touch, but with templates!

`zap` can be used as a drop-in replacement for `touch`, but it also allows you to create files pre-populated from templates. It can open newly created files with your `$EDITOR` and create intermediate directories if they do not exist.

## Installation

### Homebrew

```bash
brew tap kolja/zap https://github.com/kolja/zap
brew install kolja/zap/zap
```

### Nix

Install into your profile:
```bash
nix profile install github:kolja/zap
```

**Run without installing:**
```bash
nix run github:kolja/zap -- --version
```

## Usage

`zap` works pretty much exactly like `touch` but it has some additional features:

When you specify the `-T <template_name>`, it will look for a template file in

```
    $ZAP_CONFIG/templates/<template_name>
```

Any newly created file will be pre-populated with contents from the template.
If `ZAP_CONFIG` is not set, it defaults to `~/.config/zap/`.

You can also pass a context with the `-C` (or `--context`) to pass key-value pairs to the template.


```
Usage: zap [OPTIONS] [FILENAMES]...

Arguments:
  [FILENAMES]...

Options:
  -T, --template <TEMPLATE_NAME>  Optional template name to pre-populate the file.
                                  Templates are sourced from ~/.config/zap/<template_name>.
  -C, --context <CONTEXT>         Optional context to use when rendering the template.
                                  should contain key-value pairs in the format `foo=bar,baz=qux`.
  -p, --create-intermediate-dirs  always create intermediate directories if they do not exist
                                  (analogous to `mkdir -p`)
  -o, --open                      Open the file with your $EDITOR
  -a                              only update the access time
  -m                              only update the modification time
  -c, --no-create                 Don't create the file if it doesn't exist
  -d, --date <DATE>               pass date as human readable string (RFC3339)
  -t, --timestamp <TIMESTAMP>     pass date as POSIX compliant timestamp: [[CC]YY]MMDDhhmm[.SS]
  -r, --reference <REFERENCE>     Use access and modification times from the specified file
  -A, --adjust <ADJUST>           Adjust time [-][[hh]mm]SS
                                  the `-c` flag is implied
  -h, --help                      Print help
  -V, --version                   Print version
```

## License

MIT
