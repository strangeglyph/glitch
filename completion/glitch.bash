_glitch() {
    local i cur prev opts cmds
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${i}" in
            glitch)
                cmd="glitch"
                ;;
            
            completion)
                cmd+="__completion"
                ;;
            help)
                cmd+="__help"
                ;;
            render)
                cmd+="__render"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        glitch)
            opts=" -h -V  --help --version   render completion help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
                return 0
            fi
            case "${prev}" in
                
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
            return 0
            ;;
        
        glitch__completion)
            opts=" -h -V  --zsh --bash --fish --psh --help --version  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
                return 0
            fi
            case "${prev}" in
                
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
            return 0
            ;;
        glitch__help)
            opts=" -h -V  --help --version  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
                return 0
            fi
            case "${prev}" in
                
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
            return 0
            ;;
        glitch__render)
            opts=" -h -V -n  --help --version --number --color-shift --scan-height --scan-gap --desync-amp --desync-freq --wind-onset --wind-continue --blocks  <FILE> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
                return 0
            fi
            case "${prev}" in
                
                --number)
                    COMPREPLY=("<N>")
                    return 0
                    ;;
                    -n)
                    COMPREPLY=("<N>")
                    return 0
                    ;;
                --color-shift)
                    COMPREPLY=("<N>")
                    return 0
                    ;;
                --scan-height)
                    COMPREPLY=("<N>")
                    return 0
                    ;;
                --scan-gap)
                    COMPREPLY=("<M>")
                    return 0
                    ;;
                --desync-amp)
                    COMPREPLY=("<N>")
                    return 0
                    ;;
                --desync-freq)
                    COMPREPLY=("<M>")
                    return 0
                    ;;
                --wind-onset)
                    COMPREPLY=("<N>")
                    return 0
                    ;;
                --wind-continue)
                    COMPREPLY=("<M>")
                    return 0
                    ;;
                --blocks)
                    COMPREPLY=("<M>")
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
            return 0
            ;;
    esac
}

complete -F _glitch -o bashdefault -o default glitch
