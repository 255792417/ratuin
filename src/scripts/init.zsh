typeset -g __ratuin_last_command=""

__ratuin_preexec() {
	__ratuin_last_command="$1"
}

__ratuin_precmd() {
	local exit_code="$?"
	local cmd="$__ratuin_last_command"

	if [[ -z "$cmd" ]]; then
		return
	fi

	case "$cmd" in
		ratuin\ record*|__ratuin_preexec*|__ratuin_precmd*)
			return
			;;
	esac

	if command -v ratuin >/dev/null 2>&1; then
		ratuin record --command "$cmd" --cwd "$PWD" --exit-code "$exit_code" >/dev/null 2>&1 &!
	fi
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec __ratuin_preexec
add-zsh-hook precmd __ratuin_precmd
