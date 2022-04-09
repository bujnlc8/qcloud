complete -c qcloud -n "__fish_use_subcommand" -s h -l help -d 'Print help information'
complete -c qcloud -n "__fish_use_subcommand" -f -a "upload"
complete -c qcloud -n "__fish_use_subcommand" -f -a "download"
complete -c qcloud -n "__fish_use_subcommand" -f -a "delete"
complete -c qcloud -n "__fish_use_subcommand" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c qcloud -n "__fish_seen_subcommand_from upload" -s f -d '本地文件名称'
complete -c qcloud -n "__fish_seen_subcommand_from upload" -s k -d '对象名称, 如果未指定，和本地文件名称相同'
complete -c qcloud -n "__fish_seen_subcommand_from upload" -s h -l help -d 'Print help information'
complete -c qcloud -n "__fish_seen_subcommand_from download" -s f -d '本地保存文件名称, 如果未指定，和对象名称相同'
complete -c qcloud -n "__fish_seen_subcommand_from download" -s k -d '对象名称'
complete -c qcloud -n "__fish_seen_subcommand_from download" -s h -l help -d 'Print help information'
complete -c qcloud -n "__fish_seen_subcommand_from delete" -s k -d '对象名称'
complete -c qcloud -n "__fish_seen_subcommand_from delete" -s h -l help -d 'Print help information'
