

# ZAP

touch, but with templates!

## Installation

```bash
brew tap kolja/zap https://github.com/kolja/zap
brew install kolja/zap/zap
```

## Usage

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
