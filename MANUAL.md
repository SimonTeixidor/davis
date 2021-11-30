davis(1)

# NAME
davis - a command line interface for MPD.

# SYNOPSIS
*davis* [--verbose] [--host <host>] <command> [<args>]

# DESCRIPTION
Davis is a command line interface for MPD.

# OPTIONS
\--help
	Prints help information.
\-v, --verbose
	Enable verbose output.
\-h, --host <host>
	Specify which MPD server to talk to, can be specified using IP/hostname,
	or a label defined in the config file.
\-p, --plain
	Disable decorations in output, useful for scripting.

# DAVIS COMMANDS:
add <path>
	Add items in path to queue.

albumart -o <output> [path]
	Download album art to file specified by <output>. Davis will fetch the
	album art for the track at [path] if specified, and the currently playing
	track otherwise. If dash ('-') is specified as output, davis will write the
	album art to stdout.

clear
	Clear the current queue.

current [--no-cache]
	Display the currently playing song. If --no-cache is specified, davis
	fetches albumart from mpd and overwrites the value in cache.

del <index>
	Remove song at index from queue.

help
	Prints a brief help text.

list <tag> [query]
	List all values for tag, for songs matching query. See *QUERY*
	for details on the query format.

load <path>
	Load playlist at path to queue                    

ls [path]
	List items in path, or the root if omitted.

mv <from> <to>
	Move song in queue by index.

next
	Skip to next song in queue.

pause
	Pause playback.

play
	Continue playback from current state.

play [index]
	Start playback from index in queue.

prev
	Go back to previous song in queue.

queue
	Display the current queue.

read-comments <file>
	Read raw metadata for file. The format will depend on the format of the
	file.

search <query>
	Search the MPD database for files matching query. See *QUERY* for details on
	the format.

seek <position>             
	Seek to position. The position is expressed in [+-][[hh:]:mm]:ss format. If
	+ or - is used, the seek is done relative to the current positon.

status
	Display MPD status.

stop
	Stop playback.

toggle
	Toggle between play/pause.

update
	Update the MPD database.

# Plugins
Davis can be extended with external sub-commands. An external sub-command is
created by placing an executable file named `davis-$name` in one of the
following locations:

- `/etc/davis/bin/`
- `~/.config/davis/bin/`
- A directory in `$PATH`

An external command `davis-foo` is executed by calling `davis foo [args]`. Davis
will find the external sub command, and pass along any arguments. External sub
commands can read the `$MPD_HOST` environment variable to know which MPD server
davis is expected to speak to.

# QUERY
A query can either be a single argument in the MPD filter syntax, such as:     
	davis search '((artist == "Miles Davis") AND (album == "Kind Of Blue"))'
Or a list of arguments-pairs, each pair corresponding to a filter, such as:    
	davis search artist 'Miles Davis' album 'Kind Of Blue'         
More information on the MPD filter syntax is available at:         
	https://mpd.readthedocs.io/en/latest/protocol.html#filters  

# AUTHORS
Simon Persson <simon@flaskpost.me>
