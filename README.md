# Faramir

![Faramir gif](https://66.media.tumblr.com/407768419d9e79c38bf91e77c8a6d142/tumblr_pmpxd3VWEo1wvnem4o3_500.gif)

Faramir is a time tracking cli tool written in Rust, inspired by [Watson](https://github.com/TailorDev/Watson).

Currently in alpha, so changes will occur.

All times are stored in UTC.

## Config

By default, faramir looks for `$XDG_CONFIG_HOME`. If this isn't set, it puts `faramir-tt/` under `$HOME/.config/`.

A different config location can be specified with `-c`.

```json
{
  "data_dir": "/home/andrew/.config/faramir-tt",
  "time_format": "%Y/%m/%d %H:%M:%S",
  "full_time_format": "%Y/%m/%d %H:%M:%.3f, Day %j, Week %U",
  "timezone": "America/New_York"
}
```

* `data_dir` lets you put the actual faramir data (`faramir.db`, etc) in a different directory.
* `time_format` is used for the `add` command.
* `full_time_format` is used when the `-d / --detailed` flag is passed for some commands.
* `timezone` is a standard timezone. [Find yours here](https://docs.rs/chrono-tz/0.5.1/chrono_tz/enum.Tz.html).

## Model

A `Timer` has an `id`, an `rid` (random id), a `start` (datetime\<utc\>) and `end` (datetime\<utc\>).

`Project`s and `Tag`s have an `id` and a `name`.

Every `Timer` has a `Project`. `Project`s have many `Timer`s.

A `Timer` can have multiple `Tag`s. `Tag`s have many `Timer`s.

Associations are made through join tables, i.e. `projects_timers` and `tags_timers`.

## Commands

### add
Manually adds a timer. The format `-s` and `-e` use depends on your config's `time_format` parameter, **but** it still has to compile to a [DateTime](https://docs.rs/chrono/0.4.10/chrono/struct.DateTime.html). So `%Y.%m.%d %H:%M:%S` works but `%Y/%m/%d` doesn't.

```bash
faramir add project1 -s "2020/01/04 21:50:00" -e "2020/01/04 21:51:00" -c
```

* `-s` / `--start`
* `-e` / `--end`
* `-t` / `--tags`
* `-d` / `--duration` => not implemented yet.

### completions
Generates autocompletions for your shell. The output is in your config's `data_dir` directory.

```bash
faramir completions zsh
```

Possible values: bash, fish, zsh, powershell, elvish

### edit
Edits a timer. Do not edit `id` or `rid`, because that'll cause issues in the database.

```
faramir edit ZFhSTQgU3GtH
```

You can see a timer's id whenever you create a timer, or run `faramir log` / `faramir status`, etc.

The `$EDITOR` environment variable must be set.

### log
By default, retrieves a log of *completed* timers (that is, it doesn't include running timers).

```
% faramir log

3 timer(s) retrieved.
ZFhSTQgU3GtH - start: 2020-01-05 01:27:37.232082717 UTC, end: 2020-01-05 01:28:51.125580395 UTC
PZjIHmdC057W - start: 2020-01-05 02:50:00 UTC, end: 2020-01-05 02:51:00 UTC
3NsfWDtif6Sy - start: 2020-01-05 03:04:20.493443573 UTC, end: 2020-01-05 03:04:30.061320880 UTC
```

* `-l` / `--limit` => 10 by default.

TODO: Add other qualifiers like date range, etc

### ls
List `Project`s or `Tag`s.

```bash
% faramir ls projects

3 Project(s) found.
proj1, proj2, proj3
```

```bash
% faramir ls tags

3 Project(s) found.
tag1, tag2, tag3
```

### rename
Rename a `Project` or `Tag`.

```bash
% faramir rename p proj1 proj4

Successfully renamed project proj1 to proj4.
```

You can use `p`, `project`, or `projects` for the `Project` type.
You can use `t`, `tag`, or `tags` for the `Tag` type.

### rm
Deletes a project, tag, or timer, and associated records.

```bash
faramir rm <type> <name/id>
```

```bash
% faramir rm p proj1

Project proj1 has 4 timers associated with it. Are you sure you want to remove it?
If so, type 'y'.
y

Successfully removed project proj1.
```

* `-y` / `--yes` => Automatically deletes all related records. Dangerous!

### start
Starts a timer at the current time, UTC.

This automatically creates the passed in project and tags. Tags are optional.

```bash
% faramir start proj5 -t tag3,tag4

Successfully started timer Ga4SXq8XuZi1 for project proj5.
```

### status
Displays the status of any running timers.

TODO: make this prettier.
```bash
% faramir status

1 timer(s) found.
timer for project proj5, with id Ga4SXq8XuZi1
  Elapsed Time: 0w, 0d, 0h, 1m, 23s
  Start Time: 2020/01/05 03:38:48
```

* `-d` / `--detailed` => TODO: Show more detailed information like tags, etc.

### stop
Stops the timer if only 1 is running. Otherwise, use `-i` / `--id` to specify which timer.

```bash
% faramir stop

Stopped timer Ga4SXq8XuZi1.
```

* `-i` / `--id` => Specify a timer manually if multiple are running.

There a few more planned commands.

## License

[GPLv3](./LICENSE)
