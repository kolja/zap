

# ZAP

touch, but with templates!

## Usage

```
zap [OPTIONS] [FILENAMES]...

Arguments:
  [FILENAMES]...

Options:
  -T, --template <TEMPLATE_NAME>  Optional template name to pre-populate the file.
                                  Templates are sourced from ~/.config/zap/<template_name>.
  -C, --context <CONTEXT>         Optional context to use when rendering the template
  -o, --open                      Open the file with your $EDITOR
  -h, --help                      Print help
  -V, --version                   Print version
```

## License

MIT
