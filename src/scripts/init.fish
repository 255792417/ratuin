set -g __ratuin_last_command ""
set -g __ratuin_command_start_ms 0

function __ratuin_preexec --on-event fish_preexec
	set -g __ratuin_last_command $argv[1]
	set -g __ratuin_command_start_ms (date +%s%3N 2>/dev/null)
end

function __ratuin_postexec --on-event fish_postexec
	set -l exit_code $status
	set -l cmd "$__ratuin_last_command"
	set -l now_ms (date +%s%3N 2>/dev/null)
	set -l duration_ms 0

	if string match -qr '^[0-9]+$' -- "$__ratuin_command_start_ms"
		if string match -qr '^[0-9]+$' -- "$now_ms"
			set duration_ms (math "$now_ms - $__ratuin_command_start_ms")
			if test "$duration_ms" -lt 0
				set duration_ms 0
			end
		end
	end

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
		ratuin record --command "$cmd" --cwd "$PWD" --exit-code "$exit_code" --duration-ms "$duration_ms" >/dev/null 2>&1 &
	end
end
