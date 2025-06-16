

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
  -o, --open                      Open the file with your $EDITOR
  -a                              only update the access time
  -m                              only update the modification time
  -c, --no-create                 Don't create the file if it doesn't exist
  -h, --help                      Print help
  -V, --version                   Print version
```

## License

MIT
