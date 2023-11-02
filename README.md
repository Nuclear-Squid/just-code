just-code: A minimalist helloworld generator
============================================

A simple cli tool with a rudementary [neovim integration][#use-it-as-a-neovim-plugin]

# Use it as a cli tool

```
usage: just-code [-g|--create-git-repo] [-n|--no-editor] ([re:]file_name)+
                 -- [extra editor args]
```

This programm creates all of the files_names given as argument, copies the
template from the `~/.config/just_code.toml` file, then opens your code editor
with all file names as argument. The editor is set in "$EDITOR" env var.

The programm will replace any occurences of `$file name$` with the name of the
newly created file, keeping the same case style used in `$file name$`.
For example, if you are creating a file called `my_module.py` :

- `$file name$` will become `my module`
- `$FILE_NAME$` will become `MY_MODULE`
- `$FileName$`  will become `MyModule`…

Prefixing a file name with `re:` will remove the extention, for example :
`just-code re:hello.sh` will generate and open `hello` instead of `hello.sh`
but the file will still be handled as a `sh` file.

Extra arguments can be passed to the editor by writing them after `--`.
They will be added before the file names, for instance, if my EDITOR="nvim",
`just-code main.py module.py -- -O` will launch `nvim -O main.py module.py`

The `-n` flag prevents the editor from opening the new files.

The `-g` flag creates a new git repo

# Use it as a neovim plugin

Import it with your package manager, and configure the mappings :

```lua
require('packer').startup(function(use)
    use 'Nuclear-Squid/just-code'  -- Import the plugin
    -- Configure it : here is the default configuration for the plugin.
    -- You can use `require('just-code').setup()` to keep the default mappings.
    require('just-code').setup {
        mappings = {
            new_file_horizontal = '<leader>n'
            new_file_vertical   = '<leader>N'
        }
    }
```

You’ll also need to [install](#install) the rust programm.

# Install

You’ll need the [Rust programming langage](https://www.rust-lang.org/tools/install).
Once you do, just run this in your terminal and you’re good to go.

```
cargo install just-code
```
