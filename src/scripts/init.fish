set -g __ratuin_last_command ""

function __ratuin_preexec --on-event fish_preexec
	set -g __ratuin_last_command $argv[1]
end

function __ratuin_postexec --on-event fish_postexec
	set -l exit_code $status
	set -l cmd "$__ratuin_last_command"

	if test -z "$cmd"
		return
	end

	if string match -qr '^ratuin\s+record' -- "$cmd"
		return
	end

	if string match -qr '^__ratuin_' -- "$cmd"
		return
	end

	if command -sq ratuin
		ratuin record --command "$cmd" --cwd "$PWD" --exit-code "$exit_code" >/dev/null 2>&1 &
	end
end
