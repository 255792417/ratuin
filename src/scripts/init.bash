__ratuin_preexec() {
	__ratuin_last_command="$BASH_COMMAND"
}

__ratuin_precmd() {
	local exit_code="$?"
	local cmd="${__ratuin_last_command:-}"

	if [ -z "$cmd" ]; then
		return
	fi

	case "$cmd" in
		ratuin\ record*|__ratuin_preexec*|__ratuin_precmd*)
			return
			;;
	esac

	if command -v ratuin >/dev/null 2>&1; then
		ratuin record --command "$cmd" --cwd "$PWD" --exit-code "$exit_code" >/dev/null 2>&1 &
	fi
}

trap '__ratuin_preexec' DEBUG

if [ -n "${PROMPT_COMMAND:-}" ]; then
	PROMPT_COMMAND="__ratuin_precmd;${PROMPT_COMMAND}"
else
	PROMPT_COMMAND="__ratuin_precmd"
fi
