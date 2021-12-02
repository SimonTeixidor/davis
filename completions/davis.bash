# This script was adapted from the completion script from mpc. Credit to the
# original authors at:
# https://github.com/MusicPlayerDaemon/mpc
#
# Installation:
# - If you have system bash completion, place this in /etc/bash_completion.d or
#   source it from $HOME/.bash_completion
# - If you don't have system bash completion, source this from your .bashrc

# Escape special characters with backslashes
# Something like this should (but doesn't) also work:
# while read -r line; do printf "%q\n" "$line"; done
__escape_strings_stdin () {
	sed "s/\([&><()\";\`' ]\)/\\\\\\1/g"
}

# Read everything past the command as a single word
# This is used for filenames (they may have spaces)
__get_long_cur () {
	cur="$(echo "${COMP_LINE#*$command}" | sed 's/^ *//')"
}

# Complete long option names
_davis_long_options () {
	local IFS=$'\n'
	COMPREPLY=($(davis help | grep -o -- "$cur"'[a-z-]*=\?' | sed 's/[^=]$/& /'))
}

# Complete command names
_davis_commands () {
	local IFS=$'\n'
	hold=$(davis help 2>&1 | awk '/^ *davis [a-z]+ /{print $2" "}');
	COMPREPLY=($(compgen -W "$hold"$'\n'"status " "$cur"))
}

# Complete the add command (files)
_davis_add () {
	local IFS=$'\n'
	__get_long_cur
	COMPREPLY=($(davis tab $(eval echo "$cur") | sed -re "s%^(${cur}[^/]*/?).*%\\1%" | sort -u | __escape_strings_stdin))
}

# Complete search command (query types)
_davis_search () {
	local IFS=$'\n'
	COMPREPLY=($(IFS=' '; compgen -W "artist album title track name genre date composer performer comment disc filename any" -S ' ' "$cur"))
}

# Main completion function
_davis ()
{
	local c=1 word command

	# Skip through long options, caching host/port
	while [ $c -lt $COMP_CWORD ]; do
		word="${COMP_WORDS[c]}"
		case "$word" in
			--host=*) MPD_HOST="${word#--host=}" ;;
			--plain|-p|-v|--verbose) ;;
			*) command="$word"; break ;;
		esac
		c=$((c+1))
	done

	cur="${COMP_WORDS[COMP_CWORD]}"

	# If there's no command, either complete options or commands
	if [ -z "$command" ]; then
		case "$cur" in
			--*) _davis_long_options ;;
			-*) COMPREPLY=() ;;
			*) _davis_commands ;;
		esac
		return
	fi

	# Complete command arguments
	case "$command" in
	add)         _davis_add ;;
	clear)       ;; # no arguments
	current)     ;; # no arguments
	del)         ;; # don't complete numbers
	load)        ;;
	ls)          _davis_add ;;
	mv)          ;; # don't complete numbers
	next)        ;; # no arguments
	pause)       ;; # no arguments
	play)        ;; # don't complete numbers
	prev)        ;; # no arguments
	seek)        ;; # don't complete numbers
	status)      ;; # no arguments
	stop)        ;; # no arguments
	toggle)      ;; # no arguments
	update)      _davis_add ;;
	*)           ;;
	esac

}
complete -o nospace -F _davis davis
