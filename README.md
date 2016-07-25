# run-or-raise :frog:

`run-or-raise` is an utility for launching applications or focusing their
windows if they are already running. In more general terms, `run-or-raise` tries
to find matching windows using simple expressions and if no matching windows are
found, it executes a specified program.

## Usage

	run-or-raise CONDITION PROGRAM [ARGS...]

Operators in matching expressions are left-associative and `!` (not) binds
stronger than `&&` (and) which, in turn, binds stronger than `||` (or). Possible
properties are `class`, `name`, and `role`. If multiple windows match the
criteria, the first matching window is selected.

	# Launch or focus emacs
	run-or-raise 'class = "Emacs"' emacs

	# You can also use regular expressions for matching
	# Match windows with title ending with the string "Firefox"
	run-or-raise 'name ~ ".*Firefox$"' firefox
	
	# Conditions can also be combined to create more complex ones.
	# Match windows where the window role is browser and the class is not Emacs
	# or any window where the title doesn't contain one or more digits
	run-or-raise '! name ~ ".*\d+.*" || role = "browser" && ! class = "Emacs"' urxvt

## Example use case: xbindkeys companion

[xbindkeys](http://www.nongnu.org/xbindkeys/) is an application for launching
programs based on key events. `run-or-raise` can be combined with it to only
launch applications if they are not already running. For example, to launch or
focus Firefox by pressing `Shift+Mod4+b`, you could use the following `xbindkeys`
configuration:

	"run-or-raise 'role = \"browser\"' firefox"
	  Shift+Mod4+b
