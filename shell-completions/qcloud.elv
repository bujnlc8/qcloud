
use builtin;
use str;

set edit:completion:arg-completer[qcloud] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'qcloud'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'qcloud'= {
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand upload 'upload'
            cand download 'download'
            cand delete 'delete'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'qcloud;upload'= {
            cand -f '本地文件名称'
            cand -k '对象名称, 如果未指定，和本地文件名称相同'
            cand -h 'Print help information'
            cand --help 'Print help information'
        }
        &'qcloud;download'= {
            cand -f '本地保存文件名称, 如果未指定，和对象名称相同'
            cand -k '对象名称'
            cand -h 'Print help information'
            cand --help 'Print help information'
        }
        &'qcloud;delete'= {
            cand -k '对象名称'
            cand -h 'Print help information'
            cand --help 'Print help information'
        }
        &'qcloud;help'= {
        }
    ]
    $completions[$command]
}
