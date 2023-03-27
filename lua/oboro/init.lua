-- configurations.
--
-- plugin/<PLUGIN_ID> .... plugin id (nil or string).
-- plugins/<BUNDLE_ID> ... plugins id table.
-- cfg/<PLUGIN_ID | BUNDLE_ID> ....... config.
-- deps/<PLUGIN_ID | BUNDLE_ID> ...... depends plugin id table.
-- mods/<MODULE> ..................... plugin id table on require `<MODULE>`.
-- evs/<EVENT> ....................... plugin id table on fire `<EVENT>`.
-- fts/<FILE_TYPE> ................... plugin id table on load `<FILE_TYPE>`.
-- cmds/<COMMAND> .................... plugin id table on execute `<COMMAND>`.
-- mod_tbl ........................... configured modules.
-- ev_tbl ............................ configured events.
-- ft_tbl ............................ configured filetypes.
-- cmd_tbl ........................... configured commands.
-- lazy .............................. plugin id table to be loaded using timer.
-- startup ........................... startup config.

local loaded_plugins = {}
local loaded_mods = {}

local function configure(opt, id)
	local ok, err_msg = pcall(dofile, opt.root .. "/cfgs/" .. id)
	if not ok then
		print("[" .. id .. "] configure error: " .. (err_msg or "-- no msg --"))
	end
end

-- load plugin.
local function load(opt, id)
	if loaded_plugins[id] then
		return nil
	end
	loaded_plugins[id] = true

	for _, dep in ipairs(dofile(opt.root .. "/deps/" .. id)) do
		load(opt, dep)
	end

	local plugin = dofile(opt.root .. "/plugin/" .. id)
	if plugin ~= nil then
		vim.cmd("packadd " .. plugin)
	end

	for _, p in ipairs(dofile(opt.root .. "/plugins/" .. id)) do
		load(opt, p)
	end

	configure(opt, id)
end

return {
	setup = function(opt)
		dofile(opt.root .. "/startup")

		vim.api.nvim_create_augroup("oboro", { clear = true })

		-- setup event loader
		for _, ev in ipairs(dofile(opt.root .. "/ev_tbl")) do
			vim.api.nvim_create_autocmd({ ev }, {
				group = "oboro",
				pattern = "*",
				once = true,
				callback = function()
					for _, id in ipairs(dofile(opt.root .. "/evs/" .. ev)) do
						load(opt, id)
					end
				end,
			})
		end

		-- setup filetype loader
		for _, ft in ipairs(dofile(opt.root .. "/ft_tbl")) do
			vim.api.nvim_create_autocmd({ "FileType" }, {
				group = "oboro",
				pattern = ft,
				once = true,
				callback = function()
					for _, id in ipairs(dofile(opt.root .. "/fts/" .. ft)) do
						load(opt, id)
					end
				end,
			})
		end

		-- setup command loader
		for _, cmd in ipairs(dofile(opt.root .. "/cmd_tbl")) do
			vim.api.nvim_create_autocmd({ "CmdUndefined" }, {
				group = "oboro",
				pattern = cmd,
				once = true,
				callback = function()
					for _, id in ipairs(dofile(opt.root .. "/cmds/" .. cmd)) do
						load(opt, id)
					end
				end,
			})
		end

		-- setup module loader
		table.insert(package.loaders, 1, function(mod_name)
			if loaded_mods[mod_name] then
				return nil
			end
			loaded_mods[mod_name] = true

			local ok, ids = pcall(dofile, opt.root .. "/mods/" .. mod_name)
			if not ok then
				return nil
			end

			for _, id in ipairs(ids) do
				load(opt, id)
			end
		end)

		-- setup lazy loader
		vim.defer_fn(function()
			for _, id in ipairs(dofile(opt.root .. "/lazy")) do
				load(opt, id)
			end
		end, opt.lazy_time)
	end,
}
