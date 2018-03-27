# run-or-raise 🏃‍

[![Build Status](https://travis-ci.org/Soft/run-or-raise.svg?branch=master)](https://travis-ci.org/Soft/run-or-raise)
[![Latest Version](https://img.shields.io/crates/v/run-or-raise.svg)](https://crates.io/crates/run-or-raise)
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

`run-or-raise` is an utility for launching applications or focusing their
windows if they are already running. `run-or-raise` tries to find a matching
window using simple expressions and focus it or, if no matching windows are
found, execute a specified program.

This can be useful when combined with a tiling window manager such as i3 or a
general purpose keyboard shortcut manager such as xbindkeys that allow binding
arbitrary commands to keybindings. In such setup, one might use `run-or-raise`
to, for example, launch or focus a web browser with a single key.

## Installation

The latest version of `run-or-raise` can easily be installed using
[Cargo](https://crates.io)

	cargo install run-or-raise

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
