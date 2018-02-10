function __fish_using_command
    set cmd (commandline -opc)
    if [ (count $cmd) -eq (count $argv) ]
        for i in (seq (count $argv))
            if [ $cmd[$i] != $argv[$i] ]
                return 1
            end
        end
        return 0
    end
    return 1
end

complete -c glitch -n "__fish_using_command glitch" -s h -l help -d 'Prints help information'
complete -c glitch -n "__fish_using_command glitch" -s V -l version -d 'Prints version information'
complete -c glitch -n "__fish_using_command glitch" -f -a "render" -d 'Apply a glitch effect to images'
complete -c glitch -n "__fish_using_command glitch" -f -a "completion" -d 'Generate completion scripts'
complete -c glitch -n "__fish_using_command glitch" -f -a "help" -d 'Prints this message or the help of the given subcommand(s)'
complete -c glitch -n "__fish_using_command glitch render" -s n -l number -d 'Number of images to generate. If generating multiple images, they will form a continuous animation'
complete -c glitch -n "__fish_using_command glitch render" -l color-shift -d 'Amount of offset from original position of each color channel'
complete -c glitch -n "__fish_using_command glitch render" -l scan-height -d 'Height of each scanline'
complete -c glitch -n "__fish_using_command glitch render" -l scan-gap -d 'Height of the gap between scanlines'
complete -c glitch -n "__fish_using_command glitch render" -l desync-amp -d 'Amplitude for the desync effect'
complete -c glitch -n "__fish_using_command glitch render" -l desync-freq -d 'Frequency for the desync effect'
complete -c glitch -n "__fish_using_command glitch render" -l wind-onset -d 'Onset chance for wind effect'
complete -c glitch -n "__fish_using_command glitch render" -l wind-continue -d 'Continue chance for wind effect'
complete -c glitch -n "__fish_using_command glitch render" -l blocks -d 'Number of blocks to shift'
complete -c glitch -n "__fish_using_command glitch render" -s h -l help -d 'Prints help information'
complete -c glitch -n "__fish_using_command glitch render" -s V -l version -d 'Prints version information'
complete -c glitch -n "__fish_using_command glitch completion" -l zsh -d 'Generate zsh completion'
complete -c glitch -n "__fish_using_command glitch completion" -l bash -d 'Generate bash completion'
complete -c glitch -n "__fish_using_command glitch completion" -l fish -d 'Generate fish completion'
complete -c glitch -n "__fish_using_command glitch completion" -l psh -d 'Generate powershell completion'
complete -c glitch -n "__fish_using_command glitch completion" -s h -l help -d 'Prints help information'
complete -c glitch -n "__fish_using_command glitch completion" -s V -l version -d 'Prints version information'
complete -c glitch -n "__fish_using_command glitch help" -s h -l help -d 'Prints help information'
complete -c glitch -n "__fish_using_command glitch help" -s V -l version -d 'Prints version information'
