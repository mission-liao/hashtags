let s:save_cpo = &cpo
set cpo&vim

function! s:restore_cpo()
    let &cpo = s:save_cpo
    unlet s:save_cpo
endfunction

if exists('g:loaded_hashtags_plugin')
    call s:restore_cpo()
    finish
endif
let g:loaded_hashtags_plugin = 1

if !exists("g:hashtags_command")
    let g:hashtags_command = "hs"
endif

function! s:save_note()
    let buff=join(getline(1, '$'), "\n")
    let t = system(g:hashtags_command . " create -n '" . l:buff . "'")
    echo t
endfunction

function! s:query_notes(filter,output_format) abort
    silent let notes = system(g:hashtags_command . " query -m 'simple' -o " . a:output_format . " -f " . a:filter)
    execute "new"
    execute "put =l:notes"
    execute "setlocal nobuflisted buftype=nofile bufhidden=delete noswapfile nomodifiable readonly"
endfunction

function! s:update_note()
    let buff = join(getline(1, '$'), "\n")
    let t = system(g:hashtags_command . " update -n '" . l:buff . "'")
    echo t
endfunction

command! SaveNote call s:save_note()
command! UpdateNote call s:update_note()
command! -nargs=* QueryNotes call s:query_notes(<f-args>)

call s:restore_cpo()

