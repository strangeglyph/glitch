
@('glitch', './glitch', 'glitch.exe', '.\glitch', '.\glitch.exe', './glitch.exe') | %{
    Register-ArgumentCompleter -Native -CommandName $_ -ScriptBlock {
        param($wordToComplete, $commandAst, $cursorPosition)

        $command = '_glitch'
        $commandAst.CommandElements |
            Select-Object -Skip 1 |
            %{
                switch ($_.ToString()) {

                    'Glitch' {
                        $command += '_Glitch'
                        break
                    }

                    'render' {
                        $command += '_render'
                        break
                    }

                    'completion' {
                        $command += '_completion'
                        break
                    }

                    'help' {
                        $command += '_help'
                        break
                    }

                    default { 
                        break
                    }
                }
            }

        $completions = @()

        switch ($command) {

            '_glitch' {
                $completions = @('render', 'completion', 'help', '-h', '-V', '--help', '--version')
            }

            '_glitch_render' {
                $completions = @('-h', '-V', '-n', '--help', '--version', '--number', '--color-shift', '--scan-height', '--scan-gap', '--desync-amp', '--desync-freq', '--wind-onset', '--wind-continue', '--blocks')
            }

            '_glitch_completion' {
                $completions = @('-h', '-V', '--zsh', '--bash', '--fish', '--psh', '--help', '--version')
            }

            '_glitch_help' {
                $completions = @('-h', '-V', '--help', '--version')
            }

        }

        $completions |
            ?{ $_ -like "$wordToComplete*" } |
            Sort-Object |
            %{ New-Object System.Management.Automation.CompletionResult $_, $_, 'ParameterValue', $_ }
    }
}
