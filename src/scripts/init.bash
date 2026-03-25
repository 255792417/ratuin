__ratuin_now_ms() {
	date +%s%3N 2>/dev/null || echo 0
}

__ratuin_preexec() {
	__ratuin_last_command="$BASH_COMMAND"
	__ratuin_command_start_ms="$(__ratuin_now_ms)"
}

__ratuin_ctrl_r() {
	local selected
	selected="$(ratuin tui 2>/dev/null)"
	if [ -n "$selected" ]; then
		READLINE_LINE="$selected"
		READLINE_POINT=${#READLINE_LINE}
	fi
}

__ratuin_precmd() {
	local exit_code="$?"
	local cmd="${__ratuin_last_command:-}"
	local now_ms
	local duration_ms=0

	now_ms="$(__ratuin_now_ms)"
	if [[ "${__ratuin_command_start_ms:-}" =~ ^[0-9]+$ ]] && [[ "$now_ms" =~ ^[0-9]+$ ]]; then
		duration_ms=$((now_ms - __ratuin_command_start_ms))
		if [ "$duration_ms" -lt 0 ]; then
			duration_ms=0
		fi
	fi

	if [ -z "$cmd" ]; then
		return
	fi

	case "$cmd" in
		ratuin\ record*|__ratuin_preexec*|__ratuin_precmd*)
			return
			;;
	esac

	if command -v ratuin >/dev/null 2>&1; then
		ratuin record --command "$cmd" --cwd "$PWD" --exit-code "$exit_code" --duration-ms "$duration_ms" >/dev/null 2>&1 &
	fi
}

trap '__ratuin_preexec' DEBUG

if [ -n "${PROMPT_COMMAND:-}" ]; then
	PROMPT_COMMAND="__ratuin_precmd;${PROMPT_COMMAND}"
else
	PROMPT_COMMAND="__ratuin_precmd"
fi

bind -x '"\C-r":__ratuin_ctrl_r'
