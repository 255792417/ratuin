typeset -g __ratuin_last_command=""
typeset -g __ratuin_command_start_ms="0"

__ratuin_now_ms() {
	date +%s%3N 2>/dev/null || print -r -- 0
}

__ratuin_preexec() {
	__ratuin_last_command="$1"
	__ratuin_command_start_ms="$(__ratuin_now_ms)"
}

__ratuin_precmd() {
	local exit_code="$?"
	local cmd="$__ratuin_last_command"
	local now_ms="$(__ratuin_now_ms)"
	local duration_ms=0

	if [[ "$__ratuin_command_start_ms" == <-> && "$now_ms" == <-> ]]; then
		duration_ms=$(( now_ms - __ratuin_command_start_ms ))
		if (( duration_ms < 0 )); then
			duration_ms=0
		fi
	fi

	if [[ -z "$cmd" ]]; then
		return
	fi

	case "$cmd" in
		ratuin\ record*|__ratuin_preexec*|__ratuin_precmd*)
			return
			;;
	esac

	if command -v ratuin >/dev/null 2>&1; then
		ratuin record --command "$cmd" --cwd "$PWD" --exit-code "$exit_code" --duration-ms "$duration_ms" >/dev/null 2>&1 &!
	fi
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec __ratuin_preexec
add-zsh-hook precmd __ratuin_precmd
