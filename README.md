# r-git-fu
Some command line utilities to use in prompts and general dev lifestyle choices


## Usage
```shell
Usage: r-git-fu [OPTIONS] <COMMAND>

Commands:
  prompt
  branches
  dir-status
  help        Print this message or the help of the given subcommand(s)

Options:
  -d, --repo-path <REPO_PATH>  [default: .]
  -f, --fetch
  -t, --timeout <TIMEOUT>      [default: 2500]
  -r, --remote-status
  -p, --plain-tables
  -h, --help                   Print help
```

## Prompt use

Put this in your shell - its fast enough - around 20ms.  Could be gaster with gix - but a lot of lower level complexity for no noticeable benefit, unless you are a pigeon.

in your prompt, you can call `r-git-fu prompt`

It will give you 
```shell
(main|✔)  -> branch name, and a green tick if your worktree and local index is clean
(main↓6|●4) -> branch name, and the number of commits your branch is behind the remote ref if one exists, and number of changes/additions/removals on your local branch
```

## Directory summary
This is for when you work on lots of repos at once and need an at a glance view of what is going on (i.e. 'what was I doing before the cat interrupted my flow of thoughts...')

```shell
r-git-fu -f <some directory into which git repos are cloned>
```
