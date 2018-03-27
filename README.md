# run-or-raise 🏃‍

[![Build Status](https://travis-ci.org/Soft/run-or-raise.svg?branch=master)](https://travis-ci.org/Soft/run-or-raise)
[![Latest Version](https://img.shields.io/crates/v/run-or-raise.svg)](https://crates.io/crates/run-or-raise)
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

`run-or-raise` is a utility for launching applications or focusing their windows
if they are already running. `run-or-raise` tries to find a matching window
using simple expressions and focus it or, if no matching windows are found,
execute a specified program.

This can be useful when combined with a tiling window manager such as
[i3](https://i3wm.org) or a general purpose keyboard shortcut manager such as
[xbindkeys](http://www.nongnu.org/xbindkeys/) that allow binding arbitrary
commands to keybindings. In such setup, one might use `run-or-raise` to, for
example, launch or focus a web browser with a single key.

`run-or-raise` is designed to work with X11 based Linux systems.

## Installation

The easiest way to obtain the latest version of `run-or-raise` is to download a
precompiled, statically-linked, program binary from [GitHub releases
page](https://github.com/Soft/run-or-raise/releases). These binaries should work
on most recent Linux systems without any additional dependencies.

Alternatively, `run-or-raise` can be easily installed from the source using
[cargo](https://doc.rust-lang.org/cargo/index.html):

``` shell
$ cargo install run-or-raise
```

Compiling and running `run-or-raise` requires [libxcb](https://xcb.freedesktop.org)
library to be installed.

## Usage

``` shell
run-or-raise CONDITION PROGRAM [ARGS...]
```

When invoked, `run-or-raise` matches existing windows against `CONDITION`. If a
matching window is found, it is focused. If none of the windows fulfill the
criteria, `run-or-raise` executes `PROGRAM` passing any `ARGS` to it.

## Conditions

Conditions select windows based on their properties. In X11, each window can
have any number of properties associated with them. Examples of window
properties include *name* (typically what is visible in windows title bar),
*class* (an identifier that can be usually used to select windows of a
particular applications) and *role* (a representation of window's logical role,
eg. a web browser). The [xprop](https://www.x.org/releases/X11R7.5/doc/man/man1/xprop.1.html)
command can be used to inspect windows and their properties.

The simplest possible window matching condition simply compares one of the
properties with a value:

``` shell
run-or-raise 'name = "Spotify"' spotify
```

This would find and focus a window with the title “Spotify” or run the command
`spotify`.

Conditions support two comparison operators: `=` for exact equality comparison
with a string literal and '~' work comparing with a
[regular expression](https://en.wikipedia.org/wiki/Regular_expression).

Comparisons can be combined using logical operators: `&&` for logical *AND*,
`||` for logical *OR*, and `!` for logical *NOT*. Operators in matching
expressions are left-associative and `!` (not) binds stronger than `&&` (and)
which, in turn, binds stronger than `||` (or). Possible properties are `class`,
`name`, and `role`. Additionally, parentheses can be used to alter evaluation
order. Strings and regular expressions are written inside double quotes. If
multiple windows match the criteria, the first matching window is selected.

Bellow are some examples of how conditions can be used to select windows in
various ways:

``` shell
# Launch or focus emacs
run-or-raise 'class = "Emacs"' emacs

# You can also use regular expressions for matching
# Match windows with title ending with the string "Firefox"
run-or-raise 'name ~ ".*Firefox$"' firefox

# Conditions can also be combined to create more complex ones.
# Match windows where the window role is browser and the class is not Emacs
# or any window where the title doesn't contain one or more digits
#
# This is getting silly
run-or-raise '! name ~ ".*\d+.*" || role = "browser" && ! class = "Emacs"' urxvt
```

## Integration with External Tools

`run-or-raise` can be combined with just about any tool that allows executing
arbitrary commands in response to key events. Bellow are some hints about
configuring `run-or-raise` to work with various applications:

### xbindkeys Keyboard Shortcut Manager

[xbindkeys](http://www.nongnu.org/xbindkeys/) is an application for executing
commands based on key events. `run-or-raise` can be combined with it to only
launch applications if they are not already running. For example, to launch or
focus Firefox by pressing `Shift+Mod4+b`, you could use the following
`xbindkeys` configuration:

``` shell
"run-or-raise 'role = \"browser\"' firefox"
	Shift+Mod4+b
```

### i3 Window Manager

[i3](https://i3wm.org) is a tiling window manager that, among other things,
supports binding arbitrary commands to arbitrary keys. To bind `run-or-raise`
invocation to a key with i3, one might specify something like the following in
i3's configuration file:

``` shell
bindsym Mod4+Shift+b exec --no-startup-id \
	run-or-raise 'role = "browser"' firefox
```

### KDE Custom Shortcuts

[KDE](https://www.kde.org) allows binding arbitrary commands to key presses
using [Custom Shortcuts manager](https://docs.kde.org/trunk5/en/kde-workspace/kcontrol/khotkeys/index.html#intro).
Through this graphical configuration utility, `run-or-raise` can be used to
launch or focus applications.
