local M = {
    defaults = {
        mappings = {
            new_file_horizontal = '<leader>n',
            new_file_vertical   = '<leader>N',
        },
    }
}

function M.new_file(is_horizontal)
    return function()
        local new_file_name = vim.fn.input("New file name : ")
        os.execute('just-code -n ' .. new_file_name)
        local prefix = ""
        if is_horizontal then prefix = 'v' end
        vim.cmd(prefix .. 'split|e ' .. new_file_name)
    end
end

function M.setup(user_config)
    user_config = vim.tbl_extend("keep", user_config or {}, M.defaults)
    vim.keymap.set('n', user_config.mappings.new_file_horizontal, M.new_file(true))
    vim.keymap.set('n', user_config.mappings.new_file_vertical, M.new_file(false))
end

return M
