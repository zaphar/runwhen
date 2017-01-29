# runwhen - A utility that executes commands on user defined triggers.

## Usage

```
Runs a command on user defined triggers.

USAGE:
    runwhen --cmd <cmd> [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --cmd <cmd>    Command to run on supplied triggers

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    success    Trigger that fires if a command runs successful.
    timer      Trigger that fires on a timer.
    watch      Trigger that fires when a file or directory changes.
```

## Description

I wanted a project to learn Rust on and this one scratches an itch I've had for
a while. runwhen executes a command on a user specified trigger. There are other
utilities out there that will execute on a timer or when a file changes but I
haven't seen any that bundled all the types of triggers into one utility.
