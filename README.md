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

Put this in your shell - its fast enough - around 20ms.  Could be faster with gix - but a lot of lower level complexity for no noticeable benefit, unless you are a pigeon.

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

You get this - 
```shell
+-------------------------------------------------------+
| Repo       Branch           Dirty   Position   Remote |
+=======================================================+
| lolcat     main                                       |
| lolcat-r   main             ●1+0                      |
| r-git-fu   remote_pulling   ●8+0                      |
+-------------------------------------------------------+
```

...which largely uses the same inscrutable markers as the prompt.

You can remote pull to get a fresher remote, pass in a `-f` to fetch

```shell
$ r-git-fu -f dir-status
+-------------------------------------------------------+
| Repo       Branch           Dirty   Position   Remote |
+=======================================================+
| lolcat     main                                       |
| lolcat-r   main             ●1+0               ↑0↓1   |
| r-git-fu   remote_pulling   ●8+0                      |
+-------------------------------------------------------+
```

Now be warned - corporate VPN's can be slow, as can bodged up git sources.    You can pass in the `-t` or `--timeout` override to suit if you want to pull the remote.   If the directory status command times out (say you aren't on your VPN or the cat has knocked out your network), subsequent calls will bypass the fetch.  The idea here is you aren't waiting for an age if your directory has 50+ repos in it.   In the directory output - if its managed to fetch the repot - the markers will be green, otherwise they will be yellow.
```shell
 (remote_pulling|●8) % r-git-fu branches
+------------------------------------------------------------------+
| Last commit           Age                Branch name             |
+==================================================================+
| 2026-01-31 11:58:31   1day 4h 43m 42s    main                    |
| 2026-01-31 11:58:31   1day 4h 43m 42s    remote_pulling          |
| 2026-01-31 11:57:32   1day 4h 44m 41s    key_checking            |
| 2026-01-31 11:49:19   1day 4h 52m 54s    sorting_with_color      |
| 2026-01-30 15:13:12   2days 1h 29m 1s    sorting                 |
| 2026-01-30 15:00:01   2days 1h 42m 12s   better_broken_detection |
| 2026-01-30 12:29:42   2days 4h 12m 31s   defect_fix_a            |
| 2026-01-30 11:30:28   2days 5h 11m 45s   gix                     |
+------------------------------------------------------------------+
```
If the ascii table offends, you can override with the simple table flag.  Same goes for the directory status

```shell
(remote_pulling|●8) % r-git-fu -p branches
Last commit          Age               Branch name
2026-01-31 11:58:31  1day 4h 44m 57s   main
2026-01-31 11:58:31  1day 4h 44m 57s   remote_pulling
2026-01-31 11:57:32  1day 4h 45m 56s   key_checking
2026-01-31 11:49:19  1day 4h 54m 9s    sorting_with_color
2026-01-30 15:13:12  2days 1h 30m 16s  sorting
2026-01-30 15:00:01  2days 1h 43m 27s  better_broken_detection
2026-01-30 12:29:42  2days 4h 13m 46s  defect_fix_a
2026-01-30 11:30:28  2days 5h 13m      gix
```

## Branch summary
Lists the last commit time, how old that makes it, and the branch name on a repo.   Handy if you have a vague memory of doing something but can't quite remember



Feel free to pull and clone - will be doing the test weiner thing progressively so we get more coverage.
