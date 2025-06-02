local moonicipal = require'moonicipal'
local T = moonicipal.tasks_file()

local P, cfg = moonicipal.import(require'idan.project.rust.bevy')
cfg.crate_name = 'time_to_topple'
cfg.setup_level_editor()
cfg.extra_logging = { bevy_gltf_components = 'debug' }

function T:act()
    local buffers_loaded_in_this_tab = {}
    for _, winid in ipairs(vim.api.nvim_tabpage_list_wins(0)) do
        buffers_loaded_in_this_tab[vim.api.nvim_win_get_buf(winid)] = true
    end
    local relevant_buffers = vim.iter(vim.api.nvim_list_bufs())
    :map(function(bufnr)
        if buffers_loaded_in_this_tab[bufnr] then
            return
        end
        if not vim.api.nvim_get_option_value('modified', {buf = bufnr}) then
            return
        end
        return vim.fn.fnamemodify(vim.api.nvim_buf_get_name(bufnr), ':.')
    end):totable()
    if next(relevant_buffers) == nil then
        return
    end
    require'fzf-lua'.fzf_exec(relevant_buffers, {
        actions = {
            default = require'fzf-lua.actions'.file_edit,
        },
        fzf_opts = {
            ['--no-multi'] = '',
        }
    })

end
